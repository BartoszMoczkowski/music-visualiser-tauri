use music_visualizer_tauri_macros::DeriveErrors;
use rodio::source::Source;
use rodio::{decoder, Decoder, OutputStreamHandle};
use rustfft::num_complex::{Complex, ComplexFloat};
use rustfft::{Fft, FftPlanner};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const SENDING_BUFFER_SIZE: usize = 512;
const FFT_BUFFER_SIZE: usize = 65536;
type SendingBuffer = [i16; SENDING_BUFFER_SIZE];
type Mp3Decoder = Decoder<BufReader<File>>;

pub struct MusicAnalizer {
    sink: rodio::Sink,
    points_buffer_arc: Arc<Mutex<VecDeque<i16>>>,
    sender: Sender<SendingBuffer>,
    fft: Arc<dyn Fft<f32>>,
}

impl MusicAnalizer {
    pub fn try_new(stream_handle: &OutputStreamHandle) -> Result<Self, MusicAnalizerError> {
        let sink = rodio::Sink::try_new(&stream_handle)?;
        let points_buffer_arc = Arc::new(Mutex::new(VecDeque::from(vec![0; FFT_BUFFER_SIZE])));
        let points_buffer_arc_thread = points_buffer_arc.clone();
        let (sender, rx): (Sender<SendingBuffer>, Receiver<SendingBuffer>) = channel();
        let mut fft_planner: FftPlanner<f32> = FftPlanner::new();
        let fft = fft_planner.plan_fft_forward(FFT_BUFFER_SIZE);
        sink.set_volume(0.1);
        thread::spawn(move || loop {
            {
                match rx.recv() {
                    Ok(data) => {
                        if let Ok(mut mutex_guard) = points_buffer_arc_thread.lock() {
                            let mut temp_deque = VecDeque::from(data);
                            mutex_guard.append(&mut temp_deque);
                            mutex_guard.rotate_left(SENDING_BUFFER_SIZE);
                            mutex_guard.truncate(FFT_BUFFER_SIZE);
                        }
                    }

                    Err(_err) => {
                        println!("Thread closing");
                        break;
                    }
                }
            }
        });

        Ok(MusicAnalizer {
            sink,
            points_buffer_arc,
            sender,
            fft,
        })
    }

    pub fn play_from_path(&self, path: &str) -> Result<(), MusicAnalizerError> {
        let mp3 = File::open(path)?;
        let decoder = decoder::Decoder::new_mp3(BufReader::new(mp3))?;
        let sending_source = SendingSource::new(decoder, self.sender.clone());
        self.sink.append(sending_source);
        Ok(())
    }

    pub fn set_volume(&self, volume: f32) -> () {
        self.sink.set_volume(volume);
    }

    pub fn get_fft_buffer(&self) -> Result<VecDeque<i16>, MusicAnalizerError> {
        if let Ok(guard) = self.points_buffer_arc.lock() {
            return Ok(guard.clone());
        } else {
            return Err(MusicAnalizerError::MutexGuardError);
        }
    }

    pub fn get_frequencies(&mut self, n_freq: usize) -> Result<Vec<f32>, MusicAnalizerError> {
        if let Ok(mut guard) = self.points_buffer_arc.lock() {
            let buffer_int = guard.make_contiguous();
            if buffer_int.len() == 0 {
                return Err(MusicAnalizerError::EmptyBufferError);
            }

            let min = *buffer_int.iter().min().unwrap_or(&0) as f32;
            let max = *buffer_int.iter().max().unwrap_or(&0) as f32;

            if (max - min) == 0.0 {
                return Err(MusicAnalizerError::MaxMinZeroError);
            }

            let mut complex_buffer: Vec<Complex<f32>> = buffer_int
                .iter()
                .map(|x| Complex::new((*x as f32) / (max - min), 0.))
                .collect();

            self.fft.process(&mut complex_buffer);

            let amplitdues: Vec<f32> = complex_buffer.iter().map(|x| x.abs()).collect();

            let n: f32 = n_freq as f32;
            let start: f32 = (20.).log(3.);
            let end: f32 = (20000.).log(3.);

            let delta = (end - start) / n;
            let mut frequencies: Vec<f32> = Vec::with_capacity(n_freq);

            let mut current_end_point: f32 = (3.).powf(delta + start);
            let mut moving_average: f32 = 0.;
            let mut number_of_samples: usize = 0;
            let mut k: usize = 0;
            for i in 0..24000 {
                let sample = amplitdues[i];
                if (i as f32) < current_end_point {
                    moving_average += sample;
                    number_of_samples += 1;
                } else {
                    frequencies.push(moving_average / (number_of_samples as f32).log(3.));
                    k += 1;
                    current_end_point = (3.).powf((k as f32 + 1.) * delta + start);

                    moving_average = 0.;
                    number_of_samples = 0;
                }
            }

            let max_f = (&frequencies)
                .iter()
                .reduce(|p, x| if x > p { x } else { p })
                .unwrap_or(&1.)
                .clone();

            let scaling = max_f / (max / i16::max_value() as f32);
            let norm_f: Vec<f32> = frequencies.into_iter().map(|x| x / scaling).collect();
            return Ok(norm_f);
        } else {
            return Err(MusicAnalizerError::IoError);
        }
    }
}
#[derive(DeriveErrors, Debug, Serialize, Deserialize)]
#[errors(
    rodio::StreamError,
    rodio::PlayError,
    std::io::Error,
    rodio::decoder::DecoderError
)]
#[enum_fields(RodioSinkError, RodioStreamError, IoError, RodioDecoderError)]
pub enum MusicAnalizerError {
    RodioStreamError,
    RodioSinkError,
    RodioDecoderError,
    IoError,
    EmptyBufferError,
    MaxMinZeroError,
    MutexGuardError,
    UnimplementedError,
}
impl std::error::Error for MusicAnalizerError {}

impl std::fmt::Display for MusicAnalizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MusicAnalizerError::RodioStreamError => {
                write!(f, "Error caused by rodio stream creation")?;
                Ok(())
            }
            MusicAnalizerError::RodioSinkError => {
                write!(f, "Error when trying to create a sink")?;
                Ok(())
            }
            MusicAnalizerError::IoError => {
                write!(f, "Error when reading file")?;
                Ok(())
            }
            MusicAnalizerError::RodioDecoderError => {
                write!(f, "Error when decoding a file")?;
                Ok(())
            }
            MusicAnalizerError::EmptyBufferError => {
                write!(f, "fft buffer empty when calling fft")?;
                Ok(())
            }
            MusicAnalizerError::MaxMinZeroError => {
                write!(f, "buffer has no maximum or minimum")?;
                Ok(())
            }
            &MusicAnalizerError::MutexGuardError => {
                write!(f, "failed to acquire lock to the buffer")?;
                Ok(())
            }
            MusicAnalizerError::UnimplementedError => {
                write!(f, "not yet specified")?;
                Ok(())
            }
        }
    }
}

pub struct SendingSource {
    pub decoder: Mp3Decoder,
    pub buffer: SendingBuffer,
    pub buffer_counter: usize,
    pub sender: Sender<SendingBuffer>,
}

impl SendingSource {
    pub fn new(decoder: Mp3Decoder, sender: Sender<SendingBuffer>) -> Self {
        let buffer: [i16; 512] = [0; 512];

        SendingSource {
            decoder,
            buffer,
            buffer_counter: 0,
            sender,
        }
    }
}

impl Iterator for SendingSource {
    type Item = i16;
    fn next(&mut self) -> Option<i16> {
        if let Some(value) = self.decoder.next() {
            self.buffer[self.buffer_counter] = value;
            self.buffer_counter += 1;
            if self.buffer_counter >= 512 {
                if let Err(err) = self.sender.send(self.buffer) {
                    eprintln!("sending failed {}", err);
                };
                self.buffer_counter = 0;
            }

            return Some(value);
        } else {
            return None;
        }
    }
}

impl Source for SendingSource {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        self.decoder.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.decoder.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.decoder.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.decoder.total_duration()
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use rustfft::num_complex::ComplexFloat;

    use super::*;

    #[test]
    fn macro_test() {
        use music_visualizer_tauri_macros::DeriveErrors;
        #[derive(DeriveErrors)]
        #[errors(rodio::PlayError, MusicAnalizerError)]
        #[enum_fields(RodioSinkError, RodioStreamError)]
        enum Testing {
            RodioSinkError,
            RodioStreamError,
        }
    }

    #[test]
    fn fft_test() {
        let mut fft_planner: FftPlanner<f32> = FftPlanner::new();
        let fft = fft_planner.plan_fft_forward(FFT_BUFFER_SIZE);

        let n = FFT_BUFFER_SIZE;
        let mut buffer: Vec<Complex<f32>> = (0..n)
            .into_iter()
            .map(|x| {
                Complex::new(
                    (((x as f32) / (n as f32) * 2. * PI * 20.).sin()
                        + ((x as f32) / (n as f32) * 2. * PI * 3600.).sin()
                        + ((x as f32) / (n as f32) * 2. * PI * 16000.).sin())
                        / 3.,
                    0.,
                )
            })
            .collect();

        fft.process(&mut buffer);
        let amplitdues: Vec<f32> = buffer.iter().map(|x| x.abs()).collect();

        let n: f32 = 16.;
        let start: f32 = (20.).log(3.);
        let end: f32 = (20000.).log(3.);

        let delta = (end - start) / n;

        let mut frequencies: Vec<f32> = Vec::with_capacity(n as usize);

        let mut current_end_point: f32 = (3.).powf(delta + start);
        let mut moving_average: f32 = 0.;
        let mut number_of_samples: usize = 0;
        let mut k: usize = 0;
        for i in 1..24000 {
            let sample = amplitdues[i];
            if (i as f32) < current_end_point {
                moving_average += sample;
                number_of_samples += 1;
            } else {
                dbg!(i);
                dbg!(current_end_point);
                dbg!(moving_average);
                dbg!(number_of_samples);
                frequencies.push(moving_average / (number_of_samples as f32));
                k += 1;
                current_end_point = (3.).powf((k as f32 + 1.) * delta + start);

                moving_average = 0.;
                number_of_samples = 0;
            }
        }

        dbg!(frequencies);
        let strings: Vec<String> = amplitdues.iter().map(|x| format!("{}", x)).collect();
        let flat = strings.join(",\n");
        use std::io::Write;

        let mut file = File::create("fft_output.txt").unwrap();
        file.write_all(&flat.as_bytes()).unwrap();
    }
}

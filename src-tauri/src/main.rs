// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{sync::Mutex, thread, time::Duration, vec};

use music_visualiser_tauri::{MusicAnalizer, MusicAnalizerError};
use tauri::State;

struct MusicAnalizerState(MusicAnalizer);
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
async fn play(
    state: State<'_, Mutex<MusicAnalizerState>>,
    path: &str,
) -> Result<(), MusicAnalizerError> {
    if let Ok(guard) = state.lock() {
        guard.0.play_from_path(path)?;
        return Ok(());
    } else {
        return Err(MusicAnalizerError::MutexGuardError);
    }
}
#[tauri::command]
async fn set_volume(
    state: State<'_, Mutex<MusicAnalizerState>>,
    volume: f32,
) -> Result<(), MusicAnalizerError> {
    if let Ok(guard) = state.lock() {
        guard.0.set_volume(volume);
        return Ok(());
    } else {
        return Err(MusicAnalizerError::MutexGuardError);
    }
}
#[tauri::command]
async fn get_points(
    state: State<'_, Mutex<MusicAnalizerState>>,
) -> Result<Vec<f32>, MusicAnalizerError> {
    if let Ok(mut guard) = state.lock() {
        let vector = guard.0.get_frequencies(16)?;
        return Ok(vector);
    } else {
        return Err(MusicAnalizerError::MutexGuardError);
    }
}

fn main() {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let m = MusicAnalizer::try_new(&stream_handle).unwrap();
    thread::sleep(Duration::from_secs(10));
    tauri::Builder::default()
        .manage(Mutex::new(MusicAnalizerState(m)))
        .invoke_handler(tauri::generate_handler![
            greet, play, get_points, set_volume
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

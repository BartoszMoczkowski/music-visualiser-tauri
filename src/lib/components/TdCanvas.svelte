<script lang="ts" type="module">
    import * as THREE from "three";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/tauri";
    export let points: number[];

    onMount(() => {
        const canvas = document.querySelector("#c");
        if (canvas != null) {
            const renderer = new THREE.WebGLRenderer({
                antialias: true,
                canvas,
            });
            const fov = 75;
            const aspect = 2; // the canvas default
            const near = 0.1;
            const far = 5;
            const camera = new THREE.PerspectiveCamera(fov, aspect, near, far);
            camera.position.z = 2;
            const scene = new THREE.Scene();

            const boxsides = 0.2;
            const geometry = new THREE.BoxGeometry(
                boxsides,
                boxsides,
                boxsides,
            );

            let cube_array: THREE.Mesh[] = [];
            for (let i = 0; i < 16; i++) {
                let color = new THREE.Color();
                color.setHSL(i / 16, 1, 0.4);
                const material = new THREE.MeshPhongMaterial({
                    color: color,
                });
                const cube = new THREE.Mesh(geometry, material);
                cube.rotation.x = 0.2;
                cube.rotation.y = 0.2;

                cube.position.x = -2 + (i / 16) * 4;
                scene.add(cube);
                cube_array[i] = cube;
            }
            const color = 0xffffff;
            const intensity = 5;
            const light = new THREE.DirectionalLight(color, intensity);
            light.position.set(-2, 10, 4);
            scene.add(light);
            renderer.render(scene, camera);
            function resizeRendererToDisplaySize(
                renderer: THREE.WebGLRenderer,
            ) {
                const canvas = renderer.domElement;
                const width = canvas.clientWidth;
                const height = canvas.clientHeight;
                const needResize =
                    canvas.width !== width || canvas.height !== height;
                if (needResize) {
                    renderer.setSize(width, height, false);
                }
                return needResize;
            }
            function render(time: number) {
                const time_sec = time * 0.001;

                renderer.render(scene, camera);

                for (let i = 0; i < 16; i++) {
                    cube_array[i].position.y = points[i] - 1;
                    cube_array[i].scale.y = points[i] * 10;
                }

                const canvas = renderer.domElement;
                camera.aspect = canvas.clientWidth / canvas.clientHeight;
                camera.updateProjectionMatrix();
                if (resizeRendererToDisplaySize(renderer)) {
                    const canvas = renderer.domElement;
                    camera.aspect = canvas.clientWidth / canvas.clientHeight;
                    camera.updateProjectionMatrix();
                }
                requestAnimationFrame(render);
            }
            requestAnimationFrame(render);

            return true;
        }
    });
</script>

<canvas id="c"></canvas>

<style>
    #c {
        width: 100%;
        height: 100%;
        display: block;
    }
</style>

export function addClickForFullscreen() {
    let canvas = document.getElementsByTagName("canvas")[0];
    canvas.addEventListener("click", makeFullscreen);
}

function makeFullscreen() {
    let canvas = document.querySelector("canvas");
    canvas.requestFullscreen().then(() => {
      console.log("entered fullscreen");
    });
    canvas.requestPointerLock();
    canvas.style.cursor = "none"; 
}
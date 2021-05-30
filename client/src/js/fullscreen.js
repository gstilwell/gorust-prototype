export function addClickForFullscreen() {
    let canvas = document.getElementsByTagName("canvas")[0];
    canvas.addEventListener("click", makeFullscreen);
}

function makeFullscreen() {
    let canvas = document.querySelector("canvas");
    let cursorType = canvas.style.cursor;
    canvas.requestFullscreen().then(() => {
        canvas.requestPointerLock();
        canvas.style.cursor = "none"; 

        canvas.addEventListener("fullscreenchange", () => {
            if( !document.fullscreenElement ) {
                document.exitPointerLock();
                canvas.style.cursor = cursorType;
            }
        });
    });
}
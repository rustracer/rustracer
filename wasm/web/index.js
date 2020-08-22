import wasmInit from "../pkg/wasm.js";

const runWasm = async () => {
  // Instantiate our wasm module
  const rustWasm = await wasmInit("../pkg/wasm_bg.wasm");
  // See https://rustwasm.github.io/book/reference/debugging.html#logging-panics
  rustWasm.init_panic_hook();
  // Create a Uint8Array to give us access to Wasm Memory
  const wasmByteMemoryArray = new Uint8Array(rustWasm.memory.buffer);

  // Get our canvas element from our index.html
  const canvasElement = document.querySelector("canvas");

  const width = rustWasm.get_width();
  const height = rustWasm.get_height();
  canvasElement.width = width;
  canvasElement.height = height;
  
  // Set up Context and ImageData on the canvas
  const canvasContext = canvasElement.getContext("2d");
  const canvasImageData = canvasContext.createImageData(
    canvasElement.width,
    canvasElement.height
  );

  // Clear the canvas
  canvasContext.clearRect(0, 0, canvasElement.width, canvasElement.height);


  const drawScene = () => {

    // Generate a new scene in wasm
    rustWasm.render();
    // Create a Uint8Array to give us access to Wasm Memory
    const wasmByteMemoryArray = new Uint8Array(rustWasm.memory.buffer);

    // Pull out the RGBA values from Wasm memory
    // Starting at the memory index of out output buffer (given by our pointer)
    // 200 * 200 * 4 = checkboard max X * checkerboard max Y * number of pixel properties (R,G.B,A)
    const outputPointer = rustWasm.get_output_buffer_pointer();
    const imageDataArray = wasmByteMemoryArray.slice(
      outputPointer,
      outputPointer + width * height * 4
    );
    // Set the values to the canvas image data
    canvasImageData.data.set(imageDataArray);

    // Clear the canvas
    canvasContext.clearRect(0, 0, canvasElement.width, canvasElement.height);

    // Place the new generated checkerboard onto the canvas
    canvasContext.putImageData(canvasImageData, 0, 0);
  };

  drawScene();
};
runWasm();
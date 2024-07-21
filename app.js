import init, { Canvas } from "./pkg/rust_wasm_lib.js";

let canvas, ctx, drawingCanvas;
let isDrawing = false;
let lastX = 0;
let lastY = 0;
let currentColor = 0xff0000; // Red
let currentSize = 5;
let isEraser = false;
let currentFrame = 0;
let frames = [];
let isPlaying = false;
let playInterval;
let onionSkinEnabled = false;
let onionSkinFrames = 1;

async function initializeApp() {
  await init();

  canvas = document.getElementById("drawingCanvas");

  if (!canvas.width) canvas.width = 800;
  if (!canvas.height) canvas.height = 600;
  ctx = canvas.getContext("2d");

  drawingCanvas = Canvas.new(canvas.width, canvas.height);
  if (!drawingCanvas) {
    console.error("Failed to create Canvas object");
    return;
  }
  console.log("Canvas object created successfully");

  frames.push(drawingCanvas);

  setupEventListeners();
  setupToolbar();
  setupAnimationControls();
  setupOnionSkinControls();
  setupLayerPanel();
  updateLayerList();
  updateTimeline();

  console.log("Setup Complete");
}

function setupEventListeners() {
  canvas.addEventListener("mousedown", startDrawing);
  canvas.addEventListener("mousemove", draw);
  canvas.addEventListener("mouseup", stopDrawing);
  canvas.addEventListener("mouseout", stopDrawing);

  console.log("Event listeners set up successfully");
}

function setupToolbar() {
  document.querySelector("#toolbar").addEventListener("click", (e) => {
    if (e.target.textContent === "Pencil") {
      isEraser = false;
    } else if (e.target.textContent === "Eraser") {
      isEraser = true;
    }
  });

  const sizeControl = document.querySelector("#sizeControl input");
  sizeControl.addEventListener("input", (e) => {
    currentSize = parseInt(e.target.value);
    document.querySelector("#sizeValue").textContent = currentSize;
  });
}

function setupAnimationControls() {
  const playButton = document.querySelector(
    "#animationControls button:first-child",
  );
  playButton.addEventListener("click", togglePlay);

  const addFrameButton = document.querySelector(
    "#animationControls button:nth-child(2)",
  );
  addFrameButton.addEventListener("click", addFrame);

  const onionSkinInput = document.querySelector("#onionSkinControl input");
  onionSkinInput.addEventListener("input", (e) => {
    onionSkinFrames = parseInt(e.target.value);
    updateCanvas();
  });
}

function setupOnionSkinControls() {
  const onionSkinCheckbox = document.querySelector(
    "#onionSkinControl input[type='checkbox']",
  );
  const onionSkinFramesInput = document.querySelector(
    "#onionSkinControl input[type='number']",
  );

  onionSkinCheckbox.addEventListener("change", (e) => {
    onionSkinEnabled = e.target.checked;
    updateCanvas();
  });

  onionSkinFramesInput.addEventListener("input", (e) => {
    onionSkinFrames = parseInt(e.target.value);
    if (onionSkinEnabled) {
      updateCanvas();
    }
  });
}

function setupLayerPanel() {
  const addLayerButton = document.querySelector(
    "#layerPanel button:first-of-type",
  );
  addLayerButton.addEventListener("click", addLayer);

  const removeLayerButton = document.querySelector(
    "#layerPanel button:last-of-type",
  );
  removeLayerButton.addEventListener("click", removeLayer);
}

function startDrawing(e) {
  isDrawing = true;
  [lastX, lastY] = [e.offsetX, e.offsetY];
}

function draw(e) {
  if (!isDrawing) return;

  drawingCanvas.draw_line(
    lastX,
    lastY,
    e.offsetX,
    e.offsetY,
    currentColor,
    isEraser,
    currentSize,
  );
  drawingCanvas.commit_drawing();

  updateCanvas();

  [lastX, lastY] = [e.offsetX, e.offsetY];
}

function stopDrawing() {
  isDrawing = false;
}

function updateCanvas() {
  let onionSkinData = [];
  if (onionSkinEnabled && onionSkinFrames > 0 && frames.length > 1) {
    for (let i = 1; i <= onionSkinFrames; i++) {
      const prevFrameIndex = (currentFrame - i + frames.length) % frames.length;
      const nextFrameIndex = (currentFrame + i) % frames.length;
      if (prevFrameIndex !== currentFrame) onionSkinData.push(prevFrameIndex);
      if (nextFrameIndex !== currentFrame && nextFrameIndex !== prevFrameIndex)
        onionSkinData.push(nextFrameIndex);
    }
  }

  let compositeData;
  try {
    compositeData = drawingCanvas.get_composite_data(onionSkinData);
  } catch (error) {
    console.error("Error calling get_composite_data:", error);
    return;
  }

  if (!compositeData) {
    console.error("get_composite_data returned null or undefined");
    return;
  }

  const imageData = new ImageData(
    new Uint8ClampedArray(compositeData),
    canvas.width,
    canvas.height,
  );
  ctx.putImageData(imageData, 0, 0);
}

function togglePlay() {
  isPlaying = !isPlaying;
  if (isPlaying) {
    playInterval = setInterval(() => {
      currentFrame = (currentFrame + 1) % frames.length;
      drawingCanvas = frames[currentFrame];
      updateCanvas();
      updateTimeline();
    }, 1000 / 12); // 12 fps
  } else {
    clearInterval(playInterval);
  }
}

function addFrame() {
  const newFrame = Canvas.new(canvas.width, canvas.height);
  frames.push(newFrame);
  currentFrame = frames.length - 1;
  drawingCanvas = newFrame;
  updateCanvas();
  updateTimeline();
}

function addLayer() {
  const newLayerIndex = drawingCanvas.add_layer();
  drawingCanvas.set_active_layer(newLayerIndex);
  updateLayerList();
}

function removeLayer() {
  const activeLayer = drawingCanvas.get_active_layer();
  drawingCanvas.remove_layer(activeLayer);
  updateLayerList();
  updateCanvas();
}

function updateLayerList() {
  const layerList = document.querySelector("#layerList");
  layerList.innerHTML = "";
  for (let i = 0; i < drawingCanvas.layer_count(); i++) {
    const layerItem = document.createElement("div");
    layerItem.textContent = `Layer ${i + 1}`;
    layerItem.addEventListener("click", () => {
      drawingCanvas.set_active_layer(i);
      updateLayerList();
    });
    if (i === drawingCanvas.get_active_layer()) {
      layerItem.style.fontWeight = "bold";
    }
    layerList.appendChild(layerItem);
  }
}

function updateTimeline() {
  const timeline = document.querySelector("#timeline");
  timeline.innerHTML = "";
  frames.forEach((frame, index) => {
    const frameItem = document.createElement("div");
    frameItem.textContent = `Frame ${index + 1}`;
    frameItem.addEventListener("click", () => {
      currentFrame = index;
      drawingCanvas = frames[currentFrame];
      updateCanvas();
      updateTimeline();
    });
    if (index === currentFrame) {
      frameItem.style.fontWeight = "bold";
    }
    timeline.appendChild(frameItem);
  });
}

// Initialize the app
initializeApp().catch(console.error);

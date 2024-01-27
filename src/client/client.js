const ws = new WebSocket("ws://localhost:40001");

function loaded() {
  const conn_err = document.getElementById("conn_err");
  const content = document.getElementById("content");

  document.head.innerHTML += `
    <style>
      h1 {
        color: deepskyblue;
        font-size: 72px;
        font-family: Calibri;
        -webkit-text-stroke: 1px black;
        margin: 0;
        position: absolute;
      }
    </style>`;
}

window.addEventListener("load", loaded);

ws.addEventListener("open", () => {
  console.log("WebSocket connection established!");
  conn_err.hidden = true;
  content.hidden = false;
})

ws.addEventListener("close", () => {
  console.log("WebSocket connection closed!");
  conn_err.hidden = false;
  content.hidden = true;
});

ws.addEventListener("message", e => {
  let data = JSON.parse(e.data);
  console.log(data);

  // Clear previous child nodes
  clear_content();

  // Create elements
  let text = document.createElement("h1");
  text.appendChild(document.createTextNode(data.message_displayed));
  text.style.left = data.message_displayed_position[0] + "px";
  text.style.top = data.message_displayed_position[1] + "px";
  content.appendChild(text);

  // Play audio
  if (data.played_sound.length > 0) {
    const audio = new Audio(data.played_sound);
    audio.volume = 0.1;
    audio.play();
  }

  // Finished event creation
  if (data.type == 1) {
    // Follow notification
    window.setTimeout(finished, 2000);
  }
});

function finished() {
  ws.send("FINISHED");
  clear_content();
}

function clear_content() {
  content.childNodes.forEach((element) => {
    content.removeChild(element);
  });
}

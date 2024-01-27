const ws = new WebSocket("ws://localhost:40001");

function loaded() {
  const conn_err = document.getElementById("conn_err");
  const content = document.getElementById("content");

  // let para = document.createElement("p");
  // para.appendChild(document.createTextNode("Test paragraph."));
  // para.style.fontSize = "72px";
  // para.style.color = "deepskyblue";
  // para.style.fontFamily = "Calibri";
  // para.style.fontWeight = "bolder";
  // para.style.webkitTextStroke = "1px black";
  // content.appendChild(para);
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
  let para = document.createElement("p");
  para.appendChild(document.createTextNode(data.message_displayed));
  para.style.fontSize = "72px";
  para.style.color = "deepskyblue";
  para.style.fontFamily = "Calibri";
  para.style.fontWeight = "bolder";
  para.style.webkitTextStroke = "1px black";
  content.appendChild(para);

  // Play audio
  if (data.played_sound.length > 0) {
    const audio = new Audio(data.played_sound);
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

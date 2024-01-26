const ws = new WebSocket("ws://localhost:40001");

function loaded() {
  const conn_err = document.getElementById("conn_err");
  const content = document.getElementById("content");

  let para = document.createElement("p");
  para.appendChild(document.createTextNode("Test paragraph."));
  para.style.fontSize = "72px";
  para.style.color = "deepskyblue";
  para.style.fontFamily = "Calibri";
  para.style.fontWeight = "bolder";
  para.style.webkitTextStroke = "1px black";
  content.appendChild(para);
}

window.addEventListener("load", loaded);

ws.addEventListener("open", () => {
  console.log("WebSocket connection established!");
  conn_err.hidden = true;
  content.hidden = false;

  ws.send("Test message");
})

ws.addEventListener("close", () => {
  console.log("WebSocket connection closed!");
  conn_err.hidden = false;
  content.hidden = true;
});

ws.addEventListener("message", e => {
  console.log(e);

  let para = document.createElement("p");
  para.appendChild(document.createTextNode(e.data));
  content.insertBefore(para, content.childNodes[1]);
});

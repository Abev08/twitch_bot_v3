const ws = new WebSocket("ws://" + window.location.hostname + ":40001");
let conn_err;
let content;
let audio_player;
let video_player;

function loaded() {
  conn_err = document.getElementById("conn_err");
  content = document.getElementById("content");
  audio_player = document.createElement("audio");
  video_player = document.createElement("video");

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
      video {
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
  // console.log(data);

  // Clear previous child nodes
  clear_content();
  window.clearTimeout();

  // Play audio
  if (data.played_sound?.length > 0) {
    audio_player.pause();
    audio_player.src = data.played_sound;
    audio_player.volume = data.played_sound_volume;
    audio_player.play();
  }

  // Play video
  if (data.played_video?.length > 0) {
    video_player.pause();
    video_player.removeEventListener("ended", finished);
    video_player.src = data.played_video;
    video_player.volume = data.played_video_volume;
    video_player.style.left = data.played_video_position[0] + "px";
    video_player.style.top = data.played_video_position[1] + "px";
    video_player.width = data.played_video_size[0];
    video_player.height = data.played_video_size[1];
    video_player.play();
    content.appendChild(video_player);
  }

  // Display message
  if (data.message_displayed?.length > 0) {
    let text = document.createElement("h1");
    text.appendChild(document.createTextNode(data.message_displayed));
    text.style.left = data.message_displayed_position[0] + "px";
    text.style.top = data.message_displayed_position[1] + "px";
    content.appendChild(text);
  }

  // Finished event creation
  if (data.type == 1) {
    // Follow notification - standard 2 sec duration
    window.setTimeout(finished, 2000);
  } else if (data.type == 2) {
    // Sub notification - wait for video to finish
    video_player.addEventListener("ended", finished, false);
  } else {
    // Not recognized message? 2 sec timeout?
    window.setTimeout(finished, 2000);
  }
});

function finished() {
  ws.send("FINISHED");
  clear_content();
}

function finished(e) {
  ws.send("FINISHED");
  content.removeChild(e.target);
  clear_content();
}

function clear_content() {
  content.childNodes.forEach((element) => {
    content.removeChild(element);
  });
}

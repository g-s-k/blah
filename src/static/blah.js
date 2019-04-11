const clog = document.getElementById("chatLog");

function info(txt) {
  const el = document.createElement("h5");
  el.className = "info";
  el.innerText = txt;
  clog.appendChild(el);
}

function msg(txt, user) {
  if (user) {
    const clc = clog.children;
    (function() {
      if (clc.length > 3) {
        const stl = clc[clc.length - 4];
        if (
          stl.classList.contains("username") &&
          stl.innerText === "User " + user
        )
          return;
      }

      const name = document.createElement("i");
      name.className = "username";
      name.innerText = "User " + user;
      clog.appendChild(name);
      clog.appendChild(document.createElement("br"));
    })();
  }
  const el = document.createElement("span");
  el.className = user ? "recv" : "sent";
  el.innerText = txt;
  el.title = new Date().toISOString();
  clog.appendChild(el);
  clog.appendChild(document.createElement("br"));
  clog.scrollTop = clog.offsetHeight;
}

const ws = new WebSocket(
  "ws://" + window.location.host + window.location.pathname + "ws"
);

ws.onmessage = function(event) {
  const d = JSON.parse(event.data);
  if (d.hasOwnProperty("initial") && d.userId) {
    info("You are signed in as User #" + d.userId + ".");
  } else {
    msg(d.text, d.userId);
  }
};

function handleKeyPress(event) {
  const val = event.target.value;
  if (event.keyCode === 13 && val.trim()) {
    msg(val);
    ws.send(val);
    event.target.value = "";
  }
}

const editor = document.getElementById("chatEditor");
editor.addEventListener("keyup", handleKeyPress);
editor.focus();

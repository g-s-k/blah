const clog = document.getElementById("chatLog");

let userId;

function info(txt) {
  const el = document.createElement("h5");
  el.className = "info";
  el.innerText = txt;
  clog.appendChild(el);
}

function msg(txt, user) {
  if (user !== userId) {
    const clc = clog.children;
    (function() {
      for (let idx = clc.length - 1; idx >= 0; idx--) {
        if (clc[idx].tagName === "SPAN" && clc[idx].classList.contains("sent"))
          break;
        else if (
          clc[idx].tagName === "I" &&
          clc[idx].classList.contains("username")
        ) {
          if (clc[idx].innerText === "User " + user) {
            return;
          } else {
            break;
          }
        }
      }

      const name = document.createElement("i");
      name.className = "username";
      name.innerText = "User " + user;
      clog.appendChild(name);
      clog.appendChild(document.createElement("br"));
    })();
  }
  const el = document.createElement("span");
  el.className = user === userId ? "sent" : "recv";
  el.innerHTML = txt;
  el.title = new Date().toISOString();
  clog.appendChild(el);
  clog.appendChild(document.createElement("br"));
  clog.scrollTop = clog.offsetHeight;
}

const ws = new WebSocket(
  "ws://" + window.location.host + window.location.pathname + "ws"
);

ws.onmessage = function handleWSMessage(event) {
  const d = JSON.parse(event.data);
  if (d.hasOwnProperty("initial") && d.userId && !userId) {
    userId = d.userId;
    info("You are signed in as User #" + d.userId + ".");
  } else {
    msg(d.text, d.userId);
  }
};

const reader = new FileReader();
reader.addEventListener("load", function sendFileAsBase64() {
  ws.send(reader.result);
});
const uploader = document.querySelector('#chatInputs input[type="file"]');
uploader.addEventListener("change", function handleFileUpload(event) {
  reader.readAsDataURL(event.target.files[0]);
});
document.getElementById("chatFileUpload").addEventListener("click", function() {
  uploader.click();
});

const editor = document.getElementById("chatEditor");
editor.addEventListener("keyup", function handleKeyPress(event) {
  const val = event.target.value;
  if (event.keyCode === 13 && val.trim()) {
    ws.send(val);
    event.target.value = "";
  }
});
editor.focus();

import { initializeApp } from "https://www.gstatic.com/firebasejs/10.12.0/firebase-app.js";
import {
  getAuth,
  onAuthStateChanged,
  signOut,
} from "https://www.gstatic.com/firebasejs/10.12.0/firebase-auth.js";

const firebaseConfig = {
  apiKey:     window.DEC_FIREBASE_API_KEY     || "YOUR_API_KEY",
  authDomain: window.DEC_FIREBASE_AUTH_DOMAIN || "YOUR_AUTH_DOMAIN",
  projectId:  window.DEC_FIREBASE_PROJECT_ID  || "YOUR_PROJECT_ID",
};

const app  = initializeApp(firebaseConfig);
const auth = getAuth(app);

// --- State ---
let currentUser   = null;
let currentChatId = null;
let chats         = []; // { id, title }

// --- DOM ---
const messagesEl   = document.getElementById("messages");
const emptyHint    = document.getElementById("emptyHint");
const messageInput = document.getElementById("messageInput");
const sendBtn      = document.getElementById("sendBtn");
const newChatBtn   = document.getElementById("newChatBtn");
const chatListEl   = document.getElementById("chatList");
const userEmailEl  = document.getElementById("userEmail");
const logoutBtn    = document.getElementById("logoutBtn");

// --- Auth guard ---
onAuthStateChanged(auth, (user) => {
  if (!user) {
    window.location.href = "/login.html";
    return;
  }
  currentUser = user;
  userEmailEl.textContent = user.email || user.displayName || "";
  loadChats();
});

logoutBtn.addEventListener("click", () => signOut(auth));

// --- Chats ---
function loadChats() {
  // TODO: загрузить список чатов из Firebase через /api/chats
  // Пока пустой список
  renderChatList();
}

function renderChatList() {
  chatListEl.innerHTML = "";
  for (const chat of chats) {
    const li = document.createElement("li");
    li.textContent = chat.title;
    li.dataset.id  = chat.id;
    if (chat.id === currentChatId) li.classList.add("active");
    li.addEventListener("click", () => selectChat(chat.id));
    chatListEl.appendChild(li);
  }
}

function selectChat(id) {
  currentChatId = id;
  renderChatList();
  // TODO: загрузить сообщения чата
  messagesEl.innerHTML = "";
  emptyHint && messagesEl.appendChild(emptyHint);
}

newChatBtn.addEventListener("click", () => {
  const id = crypto.randomUUID();
  chats.unshift({ id, title: "Новый чат" });
  selectChat(id);
  renderChatList();
});

// --- Sending messages ---
sendBtn.addEventListener("click", sendMessage);

messageInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    sendMessage();
  }
});

// Auto-resize textarea
messageInput.addEventListener("input", () => {
  messageInput.style.height = "auto";
  messageInput.style.height = messageInput.scrollHeight + "px";
});

async function sendMessage() {
  const text = messageInput.value.trim();
  if (!text || !currentChatId) return;

  messageInput.value = "";
  messageInput.style.height = "auto";
  sendBtn.disabled = true;

  appendMessage("user", text);

  try {
    const idToken = await currentUser.getIdToken();

    const res = await fetch("/api/chat", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${idToken}`,
      },
      body: JSON.stringify({
        chat_id: currentChatId,
        message: text,
      }),
    });

    const data = await res.json();
    appendMessage("assistant", data.reply);
  } catch (err) {
    appendMessage("assistant", "Ошибка. Попробуй ещё раз.");
    console.error(err);
  } finally {
    sendBtn.disabled = false;
  }
}

function appendMessage(role, text) {
  emptyHint?.remove();

  const div = document.createElement("div");
  div.classList.add("msg", role);
  div.textContent = text;
  messagesEl.appendChild(div);
  messagesEl.scrollTop = messagesEl.scrollHeight;
}
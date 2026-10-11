import { initializeApp } from "https://www.gstatic.com/firebasejs/10.12.0/firebase-app.js";
import {
  getAuth,
  signInWithPopup,
  GoogleAuthProvider,
  onAuthStateChanged,
} from "https://www.gstatic.com/firebasejs/10.12.0/firebase-auth.js";

// Firebase config — значения берём из мета-тегов или window, чтобы не хранить в JS напрямую
// Вставь свои значения из Firebase Console
const firebaseConfig = {
  apiKey:     window.DEC_FIREBASE_API_KEY     || "YOUR_API_KEY",
  authDomain: window.DEC_FIREBASE_AUTH_DOMAIN || "YOUR_AUTH_DOMAIN",
  projectId:  window.DEC_FIREBASE_PROJECT_ID  || "YOUR_PROJECT_ID",
};

const app      = initializeApp(firebaseConfig);
const auth     = getAuth(app);
const provider = new GoogleAuthProvider();

const loginBtn  = document.getElementById("loginBtn");
const errorMsg  = document.getElementById("errorMsg");

// Если уже залогинен — сразу на главную
onAuthStateChanged(auth, (user) => {
  if (user) {
    window.location.href = "/index.html";
  }
});

loginBtn.addEventListener("click", async () => {
  errorMsg.textContent = "";
  try {
    await signInWithPopup(auth, provider);
    // onAuthStateChanged выше сделает редирект
  } catch (err) {
    errorMsg.textContent = "Не удалось войти. Попробуй ещё раз.";
    console.error(err);
  }
});
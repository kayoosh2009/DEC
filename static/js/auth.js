// Firebase config — заполни своими данными из Firebase Console
let auth = null;

async function initFirebase() {
    const resp = await fetch('/api/config');
    const firebaseConfig = await resp.json();
    firebase.initializeApp(firebaseConfig);
    auth = firebase.auth();
}

async function requireAuth() {
    await initFirebase();
    auth.onAuthStateChanged(user => {
        if (!user) window.location.href = '/login.html';
    });
}

async function loginWithGoogle() {
    await initFirebase();
    const provider = new firebase.auth.GoogleAuthProvider();
    auth.signInWithPopup(provider)
        .then(() => { window.location.href = '/index.html'; })
        .catch(err => { console.error('Ошибка входа:', err.message); });
}

async function logout() {
    await initFirebase();
    auth.signOut().then(() => { window.location.href = '/login.html'; });
}
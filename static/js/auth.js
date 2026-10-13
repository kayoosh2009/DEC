// Firebase config — заполни своими данными из Firebase Console
const firebaseConfig = {
    apiKey:            "YOUR_API_KEY",
    authDomain:        "YOUR_PROJECT.firebaseapp.com",
    projectId:         "YOUR_PROJECT_ID",
    storageBucket:     "YOUR_PROJECT.appspot.com",
    messagingSenderId: "YOUR_SENDER_ID",
    appId:             "YOUR_APP_ID"
};

firebase.initializeApp(firebaseConfig);
const auth = firebase.auth();

// Проверка: залогинен ли пользователь
function requireAuth() {
    auth.onAuthStateChanged(user => {
        if (!user) {
            window.location.href = '/login.html';
        }
    });
}

// Вход через Google
function loginWithGoogle() {
    const provider = new firebase.auth.GoogleAuthProvider();
    auth.signInWithPopup(provider)
        .then(result => {
            window.location.href = '/index.html';
        })
        .catch(err => {
            console.error('Ошибка входа:', err.message);
        });
}

// Выход
function logout() {
    auth.signOut().then(() => {
        window.location.href = '/login.html';
    });
}
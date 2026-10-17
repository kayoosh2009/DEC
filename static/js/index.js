// ─── State ────────────────────────────────────────────────────────────────────

let menuOpen = false;
let popupOpen = false;
let currentChatId = null;
let currentUser = null;

// ─── Init ─────────────────────────────────────────────────────────────────────

function initChat(user) {
    currentUser = user;

    // Session ID в navbar
    let id = sessionStorage.getItem('dec_id');
    if (!id) {
        id = 'DEC-' + Math.floor(1000 + Math.random() * 9000);
        sessionStorage.setItem('dec_id', id);
    }
    document.getElementById('navbarId').textContent = id;

    // Закрывать popup при клике вне
    document.addEventListener('click', e => {
        const popup = document.getElementById('logoutPopup');
        const avatar = document.getElementById('userAvatar');
        if (popupOpen && !popup.contains(e.target) && e.target !== avatar) {
            popupOpen = false;
            popup.classList.remove('visible');
        }
    });

    newChat();
}

// ─── Menu ─────────────────────────────────────────────────────────────────────

function toggleMenu() {
    menuOpen = !menuOpen;
    document.getElementById('menuIcon').classList.toggle('open', menuOpen);
    document.getElementById('sideMenu').classList.toggle('open', menuOpen);
    document.getElementById('main').classList.toggle('shifted', menuOpen);
    document.querySelector('.chat-input-bar').classList.toggle('shifted', menuOpen);
}

function toggleLogoutPopup() {
    popupOpen = !popupOpen;
    document.getElementById('logoutPopup').classList.toggle('visible', popupOpen);
}

// ─── Chat ─────────────────────────────────────────────────────────────────────

function newChat() {
    currentChatId = crypto.randomUUID();
    document.getElementById('chatMessages').innerHTML = '';
    document.getElementById('chatWelcome').classList.remove('hidden');
    document.getElementById('chatInput').value = '';
}

function autoResize(el) {
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 160) + 'px';
}

function handleKey(e) {
    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        sendMessage();
    }
}

function appendMessage(role, text, timestamp = null) {
    const container = document.getElementById('chatMessages');
    const msgId = crypto.randomUUID();
    const msgTimestamp = timestamp || new Date().toISOString();

    const wrap = document.createElement('div');
    wrap.classList.add('message', role === 'user' ? 'message-user' : 'message-ai');
    wrap.dataset.msgId = msgId;
    wrap.dataset.timestamp = msgTimestamp;
    wrap.dataset.role = role;
    wrap.dataset.content = text;

    const bubble = document.createElement('div');
    bubble.classList.add('message-bubble');
    bubble.textContent = text;

    const actions = document.createElement('div');
    actions.classList.add('message-actions');

    const starBtn = document.createElement('button');
    starBtn.classList.add('star-btn');
    starBtn.textContent = '☆';
    starBtn.onclick = () => toggleStar(starBtn, msgId, role, text, msgTimestamp);

    actions.appendChild(starBtn);
    wrap.appendChild(bubble);
    wrap.appendChild(actions);
    container.appendChild(wrap);
    container.scrollTop = container.scrollHeight;
}

async function toggleStar(btn, msgId, role, content, timestamp) {
    const isStarred = btn.classList.toggle('starred');
    btn.textContent = isStarred ? '★' : '☆';

    if (!isStarred) return;

    const idToken = await currentUser.getIdToken();

    await fetch('/chat/star', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            chat_id: currentChatId,
            msg_id: msgId,
            message: { role, content },
            timestamp,
            id_token: idToken,
            uid: currentUser.uid,
        })
    });
}

async function sendMessage() {
    const input = document.getElementById('chatInput');
    const text = input.value.trim();
    if (!text || !currentUser) return;

    // Скрываем welcome
    document.getElementById('chatWelcome').classList.add('hidden');

    input.value = '';
    input.style.height = 'auto';

    // Показываем сообщение пользователя
    appendMessage('user', text);

    // Получаем токен
    const idToken = await currentUser.getIdToken();

    const resp = await fetch('/chat/send', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            chat_id: currentChatId,
            message: text,
            timestamp: new Date().toISOString(),
            id_token: idToken,
            uid: currentUser.uid,
        })
    });

    if (!resp.ok) {
        appendMessage('ai', 'Error: ' + resp.status);
        return;
    }

    const data = await resp.json();
    appendMessage('ai', data.message);
}
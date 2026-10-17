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
    loadChatList();
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

let currentChatTitle = null;
let isFirstMessage = true;

function newChat() {
    currentChatId = crypto.randomUUID();
    currentChatTitle = null;
    isFirstMessage = true;
    document.getElementById('chatMessages').innerHTML = '';
    document.getElementById('chatWelcome').classList.remove('hidden');
    document.getElementById('chatInput').value = '';
}

async function loadChatList() {
    const idToken = await currentUser.getIdToken();
    const resp = await fetch(`/chat/list?uid=${currentUser.uid}&id_token=${encodeURIComponent(idToken)}`);
    if (!resp.ok) return;

    const chats = await resp.json();
    chats.sort((a, b) => b.timestamp.localeCompare(a.timestamp));

    const list = document.getElementById('chatList');
    list.innerHTML = '';

    for (const chat of chats) {
        const item = document.createElement('div');
        item.classList.add('chat-item');
        item.onclick = () => openChat(chat.chat_id);

        const title = document.createElement('div');
        title.classList.add('chat-item-title');
        title.textContent = chat.title;

        const date = document.createElement('div');
        date.classList.add('chat-item-date');
        date.textContent = new Date(chat.timestamp).toLocaleDateString('ru-RU', {
            day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit'
        });

        item.appendChild(title);
        item.appendChild(date);
        list.appendChild(item);
    }
}

async function openChat(chatId) {
    currentChatId = chatId;
    isFirstMessage = false;
    document.getElementById('chatMessages').innerHTML = '';
    document.getElementById('chatWelcome').classList.add('hidden');

    const idToken = await currentUser.getIdToken();
    const resp = await fetch(`/chat/messages?uid=${currentUser.uid}&id_token=${encodeURIComponent(idToken)}&chat_id=${chatId}`);
    if (!resp.ok) return;

    const msgs = await resp.json();
    for (const msg of msgs) {
        appendMessage(msg.role === 'assistant' ? 'ai' : msg.role, msg.content, msg.timestamp);
    }
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

    document.getElementById('chatWelcome').classList.add('hidden');

    input.value = '';
    input.style.height = 'auto';

    appendMessage('user', text);

    const idToken = await currentUser.getIdToken();
    const timestamp = new Date().toISOString();

    // Если первое сообщение — генерируем заголовок
    if (isFirstMessage) {
        isFirstMessage = false;
        const titleResp = await fetch('/chat/send', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                chat_id: currentChatId,
                message: `Give this chat a short title (max 5 words) based on this first message. Reply with the title only, no punctuation: "${text}"`,
                timestamp,
                id_token: idToken,
                uid: currentUser.uid,
            })
        });
        if (titleResp.ok) {
            const titleData = await titleResp.json();
            currentChatTitle = titleData.message.trim();
            await fetch('/chat/save', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    chat_id: currentChatId,
                    title: currentChatTitle,
                    timestamp,
                    id_token: idToken,
                    uid: currentUser.uid,
                })
            });
            await loadChatList();
        }
    }

    const resp = await fetch('/chat/send', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            chat_id: currentChatId,
            message: text,
            timestamp,
            id_token: idToken,
            uid: currentUser.uid,
        })
    });

    if (!resp.ok) {
        appendMessage('ai', 'Error: ' + resp.status);
        return;
    }

    const data = await resp.json();
    appendMessage('ai', data.message, data.timestamp);
    await loadChatList();
}
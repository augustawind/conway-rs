// Grid defaults.
var CHAR_ALIVE = '■';
var CHAR_DEAD = '□';

window.onload = function() {
    var isOddMsg = false;
    var addMessage = function(msg) {
        var isScrolledDown = messages.scrollHeight - messages.clientHeight <= messages.scrollTop;

        var elem = document.createElement('li');
        elem.setAttribute('class', isOddMsg ? 'message odd' : 'message');
        elem.textContent = msg;
        messages.appendChild(elem);

        isOddMsg = !isOddMsg;

        // If the message box was already scrolled down, auto-scroll down to reveal new message.
        if (isScrolledDown)
            messages.scrollTop = messages.scrollHeight - messages.clientHeight;
    };

    var gameArea = document.getElementById('game-area');

    var gridForm = document.getElementById('grid-form');
    var gridField = document.getElementById('grid-field');

    var gridOutput = document.getElementById('grid-output');
    var messages = document.getElementById('messages');

    var socket = new WebSocket('ws://localhost:3012');
    socket.onopen = function() {
        addMessage('Connected to game server.');
        socket.send('ping');
    };
    socket.onclose = function() {
        addMessage('Disconnected from game server.');
    };
    socket.onerror = function(error) {
        console.log('WebSocket Error: ' + error);
    };

    socket.onmessage = function(event) {
        var data = JSON.parse(event.data);
        if (data.status !== null)
            addMessage(data.status);
        if (data.pattern !== null)
            gridOutput.innerHTML = data.pattern
                .replace(/(\.)/g, CHAR_DEAD)
                .replace(/(x)/g, CHAR_ALIVE);
        setTimeout(function() {
            socket.send('ping');
        }, 500);
    };

    gridForm.onsubmit = function(event) {
        event.preventDefault();
        gameArea.scrollIntoView({
            block: 'start',
            inline: 'nearest',
            behavior: 'smooth'
        });

        var fontSize = parseFloat(getComputedStyle(gridOutput).getPropertyValue('font-size'));
        var width = Math.ceil(gridOutput.clientWidth / (fontSize * 0.62));
        var height = Math.ceil(gridOutput.clientHeight / (fontSize * 0.52));

        var fields = event.target.elements;
        var delay_ms = fields['tick-delay'].value;
        var secs = Math.trunc(delay_ms / 1000);
        var nanos = (delay_ms - (secs * 1000)) * 1000000;
        var delay = { secs: secs, nanos: nanos };

        var settings = {
            width: width,
            height: height,
            char_alive: CHAR_ALIVE,
            char_dead: CHAR_DEAD,
            delay: delay,
            view: fields['view'].value,
        };
        var payload = JSON.stringify({ pattern: gridField.value, settings: settings });

        var cmd = 'new-grid ' + payload;
        socket.send(cmd);
        return false;
    };

    document.querySelectorAll('#control-panel button').forEach(function(button) {
        button.onclick = function(event) {
            socket.send(event.target.value);
        };
    });
};

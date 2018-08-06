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
        });

        // Build the Settings object.
        // Use hardcoded values for `char_alive` and `char_dead`.
        var settings = { char_alive: CHAR_ALIVE, char_dead: CHAR_DEAD };

        // Compute width and height to fit containing element.
        var fontSize = parseFloat(getComputedStyle(gridOutput).getPropertyValue('font-size'));
        settings.width = Math.ceil(gridOutput.clientWidth / (fontSize * 0.61));
        settings.height = Math.ceil(gridOutput.clientHeight / (fontSize * 0.51));

        var fields = event.target.elements;

        // Fetch `delay` from form and turn it into Duration json repr. for the backend.
        var delay_ms = fields['tick-delay'].value;
        var secs = Math.trunc(delay_ms / 1000);
        var nanos = (delay_ms - (secs * 1000)) * 1000000;
        settings.delay = { secs: secs, nanos: nanos };

        // Fetch `view` from form.
        settings.view = fields['view'].value;

        // Send message.
        var payload = JSON.stringify({ pattern: gridField.value, settings: settings });
        var msg = 'new-grid ' + payload;
        socket.send(msg);
        return false;
    };

    document.querySelectorAll('#control-panel button').forEach(function(button) {
        button.onclick = function(event) {
            socket.send(event.target.value);
        };
    });
};

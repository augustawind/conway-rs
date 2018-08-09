'use strict';

/*
 * Constants.
 */
const DEFAULT_PATTERN = `
............x............
............x............
............x............
xxx......................
..x......................
.x.......................
.........................
.....xxx..xxxxx..xxx.....
.........................
.......................x.
......................x..
......................xxx
............x............
............x............
............x............
`;

const CHAR_ALIVE = '■';
const CHAR_DEAD = '□';

const MSG_CONNECTED = 'Connected';
const MSG_STATUS = 'Status';
const MSG_GRID = 'Grid';
const MSG_ERROR = 'Error';

function StatusBox() {
    let $box = document.getElementById('messages'),
        odd = false,

        add = function(msg) {
            const scrolledToEdge = $box.scrollHeight - $box.clientHeight <= $box.scrollTop;

            const elem = document.createElement('li');
            elem.setAttribute('class', odd ? 'message odd' : 'message');
            elem.textContent = msg;
            $box.appendChild(elem);

            odd = !odd;

            // Autoscroll for new messages if already at bottom.
            if (scrolledToEdge)
                $box.scrollTop = $box.scrollHeight - $box.clientHeight;
        };

    return Object.freeze({
        $box,
        add,
    });
}

window.onload = function() {
  /*
   * Get elements.
   */
    const $gameArea = document.getElementById('game-area');
    const $gridForm = document.getElementById('grid-form');
    const $gridField = document.getElementById('grid-field');
    $gridField.innerHTML = DEFAULT_PATTERN.trim();
    const $reconnectBtn = document.getElementById('reconnect-btn');
    const $grid = document.getElementById('grid-area');
    const messages = StatusBox();

  /*
   * Setup WebSocket.
   */
    const socket = new WebSocket('ws://localhost:3012');
    socket.onclose = function() {
        messages.add('Disconnected from game server.');
    };
    socket.onerror = function(error) {
        console.log('Error communicating with game server: ' + error);
    };
    socket.onmessage = function(event) {
        const msg = JSON.parse(event.data);
        switch (msg.kind) {
        case MSG_CONNECTED:
            messages.add('Connected to game server.');
            break;
        case MSG_STATUS:
            messages.add(msg.content);
            break;
        case MSG_GRID:
            $grid.innerHTML = msg.content.trim()
                .replace(/(\.)/g, CHAR_DEAD)
                .replace(/(x)/g, CHAR_ALIVE);
            break;
        case MSG_ERROR:
            messages.add('ERROR: ' + msg.content);
            break;
        }
        setTimeout(function() {
            socket.send('ping');
        }, 500);
    };

  /*
   * Setup grid form.
   */
    $gridForm.onsubmit = function(event) {
        event.preventDefault();
        $gameArea.scrollIntoView({
            block: 'start',
            inline: 'nearest',
        });

      // Build the Settings object.
      // Use hardcoded values for `char_alive` and `char_dead`.
        const settings = { char_alive: CHAR_ALIVE, char_dead: CHAR_DEAD };

      // Compute width and height to fit containing element.
        const fontSize = parseFloat(getComputedStyle($grid).getPropertyValue('font-size'));
        settings.width = Math.ceil($grid.clientWidth / (fontSize * 0.61));
        settings.height = Math.ceil($grid.clientHeight / (fontSize * 0.51));

        const fields = event.target.elements;

      // Fetch `delay` from form and turn it into Duration json repr. for the backend.
        const delay_ms = fields['tick-delay'].value;
        const delay_secs = Math.trunc(delay_ms / 1000);
        const delay_nanos = (delay_ms - (delay_secs * 1000)) * 1000000;
        settings.delay = { secs: delay_secs, nanos: delay_nanos };

      // Fetch `view` from form.
        settings.view = fields['view'].value;

      // Send message.
        const payload = JSON.stringify({ pattern: $gridField.value, settings: settings });
        const msg = 'new-grid ' + payload;

        if (socket.readyState !== socket.OPEN) {
            messages.add('Disconnected from game server.');
            reconnect();
        }
        socket.send(msg);
    };

    const reconnect = function(silent) {
        if (!silent)
            messages.add('Attempting to reconnect...');
      // FIXME -> once WebSocket loses connection a new one must be created
      // socket.io will fix this
        socket.dispatchEvent(new Event('open'));
    };

  /*
   * Reconnect button
   */
    $reconnectBtn.onclick = function() {
        if (socket.readyState === socket.OPEN)
            messages.add('Already connected to game server.');
        reconnect(true);
    };

  /*
   * Setup control panel.
   */
    document.querySelectorAll('#control-panel button').forEach(function(button) {
        button.onclick = function(event) {
            if (socket.readyState !== socket.OPEN) {
                messages.add('Disconnected from game server.');
            }
            socket.send(event.target.value);
        };
    });
};

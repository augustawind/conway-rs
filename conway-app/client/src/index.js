'use strict';

/*
 * Constants.
 */
const WEBSOCKET_URL = 'ws://localhost:3012';

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

function GameClient(spec) {
    let { status, $grid } = spec,
        $socket = new WebSocket(WEBSOCKET_URL),
        send = function(msg) {
            return $socket.send(msg);
        },
        connected = function() {
            return $socket.readyState === $socket.OPEN;
        },
        reconnect = function() {
            $socket = new WebSocket(WEBSOCKET_URL);
        };

    Object.assign($socket, {
        onclose() {
            status.add('Disconnected from game server.');
        },
        onerror(error) {
            console.log('Error communicating with game server: ' + error);
        },
        onmessage(event) {
            const msg = JSON.parse(event.data);
            switch (msg.kind) {
            case MSG_CONNECTED:
                status.add('Connected to game server.');
                break;
            case MSG_STATUS:
                status.add(msg.content);
                break;
            case MSG_GRID:
                $grid.innerHTML = msg.content.trim()
                    .replace(/(\.)/g, CHAR_DEAD)
                    .replace(/(x)/g, CHAR_ALIVE);
                break;
            case MSG_ERROR:
                status.add('ERROR: ' + msg.content);
                break;
            }
            setTimeout(function() {
                $socket.send('ping');
            }, 500);
        }
    });

    return Object.freeze({
        $socket,
        send,
        connected,
        reconnect
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
    const status = StatusBox();

  /*
   * Setup WebSocket.
   */
    let client = GameClient({ status, $grid });

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

        if (!client.connected()) {
            status.add('Disconnected from game server.');
            client.reconnect();
        }
        client.send(msg);
    };

  /*
   * Reconnect button
   */
    $reconnectBtn.onclick = function() {
        if (client.connected())
            status.add('Already connected to game server.');
        else
            client.reconnect();
    };

  /*
   * Setup control panel.
   */
    document.querySelectorAll('#control-panel button').forEach(function(button) {
        button.onclick = function(event) {
            if (client.readyState !== client.OPEN) {
                status.add('Disconnected from game server.');
            }
            client.send(event.target.value);
        };
    });
};

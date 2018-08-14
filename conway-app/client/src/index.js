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

const DEFAULT_SETTINGS = Object.freeze({
    char_alive: '■',
    char_dead: '□',
    view: 'fixed'
});

const MSG_CONNECTED = 'Connected';
const MSG_STATUS = 'Status';
const MSG_GRID = 'Grid';
const MSG_ERROR = 'Error';

function CMD(name, f = null) {
    if (!f)
        return () => name;
    return (...args) => Object.freeze({ [name]: f(...args) });
}

const CMD_MAP = Object.freeze({
    ping: CMD('Ping'),
    step: CMD('Step'),
    play: CMD('Play'),
    pause: CMD('Pause'),
    toggle: CMD('Toggle'),
    scroll: CMD('Scroll', (dx, dy) => [parseInt(dx), parseInt(dy)]),
    center: CMD('Center'),
    newGrid: CMD('NewGrid', (grid) => grid),
    restart: CMD('Restart')
});

const KEYBOARD_SHORTCUTS = Object.freeze({
    ' ': (client) => client.send(CMD_MAP.toggle()),
});

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
            return $socket.send(JSON.stringify(msg));
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
            let then = Date.now(),
                delayMs = 500,
                messages = JSON.parse(event.data);

            messages.forEach(function(msg) {
                switch (msg.kind) {
                case MSG_CONNECTED:
                    status.add('Connected to game server.');
                    break;
                case MSG_STATUS:
                    status.add(msg.content);
                    break;
                case MSG_GRID:
                    $grid.innerHTML = msg.content.trim();
                    break;
                case MSG_ERROR:
                    status.add('Error: ' + msg.content);
                    break;
                }
            });

            delayMs -= (Date.now() - then);
            setTimeout(function() {
                send(CMD_MAP.ping());
            }, delayMs);
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
        if (!client.connected()) {
            status.add('Disconnected from game server. Reconnecting...');
            client.reconnect();
            return;
        }

        event.preventDefault();
        $gameArea.scrollIntoView({
            block: 'start',
            inline: 'nearest',
        });

        // Build the Settings object.
        const fields = event.target.elements;

        // Fetch `delay` from form and turn it into Duration json repr. for the backend.
        const delay_ms = fields['tick-delay'].value;
        const delay_secs = Math.trunc(delay_ms / 1000);
        const delay_nanos = (delay_ms - (delay_secs * 1000)) * 1000000;

        // Compute width and height to fit containing element.
        const fontSize = parseFloat(getComputedStyle($grid).getPropertyValue('font-size'));
        const width = Math.ceil($grid.clientWidth / (fontSize * 0.61));
        const height = Math.ceil($grid.clientHeight / (fontSize * 0.51));

        const settings = Object.assign({
            delay: {
                secs: delay_secs,
                nanos: delay_nanos
            }
        }, DEFAULT_SETTINGS);

        // Send message.
        const payload = {
            pattern: $gridField.value,
            settings: settings,
            bounds: [width, height],
        };
        client.send(CMD_MAP.newGrid(payload));
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
            if (!client.connected())
                status.add('Disconnected from the game server. Start a new game to reconnect.');

            let [name, params] = event.target.value.split(':', 2),
                makeCmd = CMD_MAP[name],
                cmd = params
                    ? makeCmd(...params.split(','))
                    : makeCmd();

            client.send(cmd);
        };
    });
    document.addEventListener('keyup', function(event) {
        const handleKey = KEYBOARD_SHORTCUTS[event.key];
        if (handleKey) {
            event.preventDefault();
            handleKey(client);
        }
    });
};

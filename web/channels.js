import {Duplex} from 'stream';


class Channel extends Duplex {
    constructor(name) {
        super({ objectMode: true });
        this.name = name;

        let socket = this.socket = new WebSocket(`ws://${location.host}/${name}`);
        socket.binaryType = 'arraybuffer';

        socket.onopen = () => {
            console.log(`${name} onopen`, arguments);
            this.emit('connect');
        };

        socket.onclose = (e) => {
            console.log(`${name} onclose`, arguments);
            if (!e.wasClean)
                this.emit('error', e);

            this.push(null);
        };

        socket.onmessage = (e) => {
            if (!this.push(this._unpack(e.data))) {
                socket.onclose = null;
                socket.close();
            }
        };

        socket.onerror = () => {
            console.log(`${name} onerror`, arguments);
            socket.close();
        };
    }

    _pack(data) { return data; }
    _unpack(raw) { return raw; }

    _read() {}

    _write(chunk, enc, cb) {
        if (this.socket.readyState === 1)
            this.socket.send(this._pack(chunk));

        cb();
    }
}

export class VideoChannel extends Channel {
    _unpack(raw) {
        return new Uint8Array(raw);
    }
}

export class AttitudeChannel extends Channel {
    _unpack(raw) {
        let dv = new DataView(raw);
        let w = dv.getFloat32(0, true);
        let x = dv.getFloat32(4, true);
        let y = dv.getFloat32(8, true);
        let z = dv.getFloat32(12, true);

        return [w, x, y, z];
    }
}

export class SysInfoChannel extends Channel {
    _unpack(raw) {
        let dv = new DataView(raw);

        return {
            freeMem: dv.getUint8(0) / 255,
            availMem: dv.getUint8(1) / 255,
            cpu: dv.getUint8(2) / 255,
            loadavg: [dv.getUint8(3) / 100, dv.getUint8(4) / 100, dv.getUint8(5) / 100],
            temp: dv.getInt8(6)
        };
    }
}

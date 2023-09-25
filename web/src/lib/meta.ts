export class MetaPacker {
    data: number[];

    constructor() {
        this.data = [];
    }

    push_var_int(num: number) {
        while (num !== 0) {
            let b = num & 0x7F;
            num >>= 7;

            if (num !== 0) {
                b |= 0x80;
            }

            this.data.push(b);
        }
    }

    push_str(str: string) {
        const view = new TextEncoder().encode(str);

        this.data.push(...view);
    }

    build(): Uint8Array {
        return new Uint8Array(this.data);
    }
}

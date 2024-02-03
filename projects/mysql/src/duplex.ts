/**
 * Local byte duplex for MySQL wire sessions (package-private).
 * Not a public cross-driver Transport product.
 */

export type ByteHandlers = {
    onMessage: (bytes: Uint8Array) => void;
    onClose?: (info: { code: number; reason: string }) => void;
    onError?: (error: unknown) => void;
};

export type ByteDuplex = {
    readonly connected: boolean;
    send(bytes: Uint8Array): void;
    close(code?: number, reason?: string): void;
};

/// Convert Buffer to Uint8Array
export function bufferToArray(buffer: Buffer): number[] {
    const nums = [];
    for (let i = 0; i < buffer.length; i++) {
      nums.push(buffer[i]);
    }
    return nums;
}
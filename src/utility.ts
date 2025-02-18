/** without extension */
export const fileName = (path: string) => {
    const match = path.match(/([^\\]+)\.[a-zA-Z0-9]+$/)
    if (match) {
        return match[0]
    } else {
        return undefined
    }
}
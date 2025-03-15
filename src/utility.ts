/** without extension or framerate */
export const fileName = (path: string) => {
    const match = path.match(/([^\\]+)\.[a-zA-Z0-9]+$/)
    if (match) {
        const split =  match[1].split('.')
        if (split.length > 1 && split[split.length-1].trim().length !== 0 && !isNaN(Number(split[split.length-1]))) {
            return split.slice(0, -1).join('_')
        }
        return match[1]
    } else {
        return undefined
    }
}
import { open, OpenDialogOptions } from '@tauri-apps/plugin-dialog';

type Props = {
    value: string[],
    htmlId: string,
    name: string,
    onConfirm: (paths: string[]) => void
    tooltip?: string,
    options?: OpenDialogOptions
}
const PathSelector = ({ htmlId, name, options, tooltip, value, onConfirm }: Props) => {
    const onButtonPress = async () => {
        const file: string | string[] | null = await open(options)
        if (file) {
            if (typeof file === 'string') {
                onConfirm([file])
            } else {
                onConfirm(file)
            }
        }
    }

    return (
        <div style={{display: 'flex', alignItems: 'flex-end', marginBottom: 6, gap: 10}}>
            <div className="field-row-stacked">
                <label htmlFor={htmlId}>{name}</label>
                <input title={tooltip} id={htmlId} type="text" value={value.join(', ')} onChange={e => {if (e.target.value === '') onConfirm([])}} autoComplete='off' />
            </div>
            <button onClick={onButtonPress}>Select File</button>
        </div>
    )
}

export default PathSelector

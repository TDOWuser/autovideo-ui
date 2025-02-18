import { open, OpenDialogOptions } from '@tauri-apps/plugin-dialog';
import { useState } from 'react';

type Props = {
    htmlId: string,
    name: string,
    tooltip?: string,
    options?: OpenDialogOptions
}
const PathSelector = ({ htmlId, name, options, tooltip }: Props) => {
    const [value, setValue] = useState<string[]>([])

    const onButtonPress = async () => {
        const file: string | string[] | null = await open(options)
        if (file) {
            if (typeof file === 'string') {
                setValue([file])
            } else {
                setValue(file)
            }
        }
    }

    return (
        <div style={{display: 'flex', alignItems: 'flex-end', marginBottom: 6, gap: 10}}>
            <div className="field-row-stacked">
                <label htmlFor={htmlId}>{name}</label>
                <input title={tooltip} id={htmlId} type="text" value={value.join(', ')} onChange={e => {if (e.target.value === '') setValue([])}} />
            </div>
            <button onClick={onButtonPress}>Select File</button>
        </div>
    )
}

export default PathSelector

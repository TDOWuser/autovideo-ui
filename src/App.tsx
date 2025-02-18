import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "7.css"
import "./App.css"
import { getCurrentWindow } from "@tauri-apps/api/window";
import PathSelector from "./components/path-selector.component";
import { fileName } from "./utility";
import { listen } from "@tauri-apps/api/event";

const appWindow = getCurrentWindow()
document.getElementById('minimize-button')?.addEventListener('click', () => appWindow.minimize())
document.getElementById('close-button')?.addEventListener('click', () => appWindow.close())

window.addEventListener('contextmenu', e => e.preventDefault())

function App() {
    const [active, setActive] = useState(false)
    const [progress, setProgress] = useState({current: 0, max: 1})
    const [activeTab, setActiveTab] = useState<'general' | 'log'>('general')
    const [log, setLog] = useState<string[]>([])

    const [modName, setModName] = useState('')
    const [selectedGenerate, setSelectedGenerate] = useState<'esp' | 'script'>('esp')
    const [size, setSize] = useState(512)
    const [fps, setFps] = useState(10)

    const [inputs, setInputs] = useState<string[]>([])
    const [esp, setEsp] = useState<string>()
    const [desp, setDesp] = useState<string>()

    const [espName, setEspName] = useState('')
    const [tvRecord, setTvRecord] = useState('')
    const [prRecord, setPrRecord] = useState('')
    const [driveInEspName, setDriveinEspName] = useState('')

    const [shortNames, setShortNames] = useState(true)
    const [keepAspectRatio, setKeepAspectRatio] = useState(true)

    let namesTooLong
    if (shortNames) {
        const shortNamesList = inputs.map(input => fileName(input)?.slice(0, 10))
        namesTooLong = inputs.some((input, index) => shortNamesList.indexOf(fileName(input)?.slice(0, 10)) !== index)
    } else {
        namesTooLong = inputs.some(i => fileName(i)!.length > 10)
    }

    const inputValid = inputs.length > 0
        && !namesTooLong
        && modName.length > 0
        && (selectedGenerate === 'script' ? (espName.length > 0 && tvRecord.length > 0 && prRecord.length > 0) : true)

    const onStart = async () => {
        setActive(true)
        const unlisten = await listen<{ current: number, max: number }>('listener', (event) => setProgress(event.payload))
        await invoke('convert_files', {
            inputs,
            inputEsp: esp,
            inputEspDriveIn: desp,
            modName: 'test',
            inputFramerate: fps,
            shortNames,
            videoName: undefined,
            size,
            keepAspectRatio,
            scriptInfo: selectedGenerate === 'script' ? {
                esp_name: espName,
                tv_record: tvRecord,
                pr_record: prRecord,
                di_esp_name: driveInEspName
            } : undefined
        })
        unlisten()
        setActive(false)
    }

    return (
        <div className="window-body has-space">
            <section className="tabs">
                <menu role="tablist" aria-label="Tabs Template">
                    <button role="tab" aria-controls="tab-A" aria-selected={activeTab === 'general'} onClick={() => setActiveTab('general')}>General</button>
                    <button role="tab" aria-controls="tab-B" aria-selected={activeTab === 'log'} onClick={() => setActiveTab('log')}>Log</button>
                </menu>
                <article role="tabpanel" id="tab-A" hidden={activeTab !== 'general'}>
                    <div style={{display: 'flex', justifyContent: 'space-between'}}>
                        <div>
                            <div className="field-row-stacked" style={{marginBottom: 6}}>
                                <label htmlFor="mod-name">Mod Name*</label>
                                <input autoComplete="off" id="mod-name" type="text" value={modName} onChange={e => {if (e.target.value.length <= 10) setModName(e.target.value)}} />
                            </div>
                            <PathSelector
                                value={inputs}
                                onConfirm={setInputs}
                                htmlId="input-path"
                                name="Input Path*"
                                options={{multiple: true, filters: [{name: 'Video', extensions: ['mp4', 'mkv', 'avi', 'gif', 'webp']}]}}
                                tooltip={`Path to video or folder of videos to convert.\nNames of video files will be used to name the holotapes. In case of single video file, name can be overwritten using "-n".`}
                            />
                            {selectedGenerate === 'esp' && <>
                                <PathSelector
                                    value={esp ? [esp] : []}
                                    onConfirm={value => setEsp(value.length > 0 ? value[0] : undefined)}
                                    htmlId="esp-path"
                                    name="ESP Path"
                                    options={{filters: [{name: 'CreationKit ESP', extensions: ['esp']}]}}
                                />
                                <PathSelector
                                    value={desp ? [desp] : []}
                                    onConfirm={value => setDesp(value.length > 0 ? value[0] : undefined)}
                                    htmlId="desp-path"
                                    name="DriveIn ESP Path"
                                    options={{filters: [{name: 'CreationKit ESP', extensions: ['esp']}]}}
                                />
                            </>}
                            {selectedGenerate === 'script' && <>
                                <div className="field-row-stacked">
                                    <label htmlFor="esp-name-input">ESP Name*</label>
                                    <input id="esp-name-input" type="text" value={espName} onChange={e => setEspName(e.target.value)} />
                                </div>
                                <div className="field-row-stacked">
                                    <label htmlFor="tv-record-input">TV Record*</label>
                                    <input id="tv-record-input" type="text" value={tvRecord} onChange={e => setTvRecord(e.target.value)} />
                                </div>
                                <div className="field-row-stacked">
                                    <label htmlFor="pr-record-input">Projector Record*</label>
                                    <input id="pr-record-input" type="text" value={prRecord} onChange={e => setPrRecord(e.target.value)} />
                                </div>
                                <div className="field-row-stacked">
                                    <label htmlFor="di-esp-input">DriveIn ESP Name</label>
                                    <input id="di-esp-input" type="text" value={driveInEspName} onChange={e => setDriveinEspName(e.target.value)} />
                                </div>
                            </>}
                        </div>
                        <div>
                            <fieldset>
                                <legend>ESP / Script</legend>
                                <div className="field-row">
                                    <input id="selector-esp" type="radio" name="esp" onChange={() => setSelectedGenerate('esp')} checked={selectedGenerate === 'esp'}/>
                                    <label htmlFor="selector-esp">Generate esp file</label>
                                </div>
                                <div className="field-row">
                                    <input id="selector-script" type="radio" name="script" onChange={() => setSelectedGenerate('script')} checked={selectedGenerate === 'script'}/>
                                    <label title={`For advanced users. Generates a FO4Edit script to add video records to existing esp. No esps will be generated\nUseful for when you already have an existing VotW esp, either a full one made by autovideo or one you made yourself`} htmlFor="selector-script">Generate script</label>
                                </div>
                            </fieldset>
                            <div style={{display: 'flex', gap: 10}}>
                                <div>
                                    <label htmlFor="size-select" style={{marginRight: 5}}>Size</label>
                                    <select id="size-select" value={size} onChange={e => setSize(Number(e.target.value))}>
                                        {[128, 256, 512, 1024].map(option => (
                                            <option key={option}>{option}</option>
                                        ))}
                                    </select>
                                </div>
                                <div>
                                    <label htmlFor="fps-select" style={{marginRight: 5}}>FPS</label>
                                    <input id="fps-select" type="number" min={1} max={60} value={fps} onChange={e => {
                                        const nr = Number(e.target.value)
                                        if (nr <= 60 && nr >= 1) {
                                            setFps(nr)
                                        }
                                    }} />
                                </div>
                            </div>
                            <fieldset>
                                <legend>Options</legend>
                                <div className="field-row">
                                    <input checked={shortNames} onChange={() => setShortNames(b => !b)} type="checkbox" id="short-names" />
                                    <label htmlFor="short-names">Short names</label>
                                </div>
                                <div className="field-row">
                                    <input checked={keepAspectRatio} onChange={() => setKeepAspectRatio(b => !b)} type="checkbox" id="keep-aspect-ratio" />
                                    <label htmlFor="keep-aspect-ratio">Keep aspect ratio</label>
                                </div>
                            </fieldset>
                        </div>
                    </div>
                    <div style={{display: 'flex', gap: 10, alignItems: 'center', marginTop: 6}}>
                        <div role="progressbar" className={active ? (progress.current === 0 ? 'marquee' : 'animate') : ''} style={{width: '100%'}}>
                            <div style={{width: `${progress.current/progress.max*100}%`}}></div>
                        </div>
                        <button onClick={onStart} disabled={!inputValid || active}>START</button>
                    </div>
                </article>
                <article role="tabpanel" id="tab-B" hidden={activeTab !== 'log'}>
                    {log.map(message => (
                        <p style={{padding: 0, margin: 0}}>{message}</p>
                    ))}
                </article>
            </section>
        </div>
    );
}

export default App;

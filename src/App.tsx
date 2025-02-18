import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "7.css"
import "./App.css"
import { getCurrentWindow } from "@tauri-apps/api/window";
import PathSelector from "./components/path-selector.component";

const appWindow = getCurrentWindow()
document.getElementById('minimize-button')?.addEventListener('click', () => appWindow.minimize())
document.getElementById('close-button')?.addEventListener('click', () => appWindow.close())

function App() {
    const [selectedGenerate, setSelectedGenerate] = useState<'esp' | 'script'>('esp')
    const [size, setSize] = useState('512')
    const [fps, setFps] = useState(10)

    // async function greet() {
    //     // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    //     setGreetMsg(await invoke("greet", { name }));
    // }

    return (
        <div className="window-body has-space">
            <section className="tabs">
                <menu role="tablist" aria-label="Tabs Template">
                    <button role="tab" aria-controls="tab-A" aria-selected="true">General</button>
                    <button role="tab" aria-controls="tab-B" aria-selected="false">Info</button>
                </menu>
                <article role="tabpanel" id="tab-A">
                    <PathSelector
                        htmlId="input-path"
                        name="Input Path"
                        options={{multiple: true, filters: [{name: 'Video', extensions: ['mp4', 'mkv', 'avi']}]}}
                        tooltip={`Path to video or folder of videos to convert.\nNames of video files will be used to name the holotapes. In case of single video file, name can be overwritten using "-n".`}
                    />
                    <PathSelector htmlId="esp-path" name="ESP Path" options={{filters: [{name: 'CreationKit ESP', extensions: ['esp']}]}} />
                    <PathSelector htmlId="desp-path" name="DriveIn ESP Path" options={{filters: [{name: 'CreationKit ESP', extensions: ['esp']}]}} />
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
                    <div>
                        <label htmlFor="size-select" style={{marginRight: 5}}>Size</label>
                        <select id="size-select" value={size} onChange={e => setSize(e.target.value)}>
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
                    <div role="progressbar" className="marquee"></div>
                </article>
                <article role="tabpanel" id="tab-B" hidden>Tab B is active</article>
            </section>
        </div>
    );
}

export default App;

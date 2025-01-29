import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";

function PreloadMainPage() {
    useEffect(() => {
        async function run_cshunter() {
            await invoke("run_main_window_and_close_preload");
        }
        
        run_cshunter();
    }, []);

    return (<></>)
}

export default PreloadMainPage;
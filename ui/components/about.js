import { LitElement, html } from "lit";

export class About extends LitElement {
    static properties = {
        version: { type: String },
        git: { type: String }
    };

    constructor() {
        super();
        this.version = "n/a";
        this.git = "n/a";
        this.updateVersion();
    }

    async updateVersion() {
        const versionResponse = await fetch("version");
        const json = await versionResponse.json();
        this.version = json.version;
        this.git = json.git;
    }

    render() {
        return html`
            <p>
                This DMARC Report Viewer is an open source application.<br>
                You can find the source code and license on Github:
                <a href="https://github.com/cry-inc/dmarc-report-viewer" target="_blank">github.com/cry-inc/dmarc-report-viewer</a>.
            </p>
            <p>
                Version: <b>${this.version}</b><br>
                Git-Hash: <b>${this.git}</b>
            </p>
        `;
    }
}

customElements.define("dmarc-about", About);

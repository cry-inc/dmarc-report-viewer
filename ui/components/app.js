import { LitElement, html, css } from "lit";

export class App extends LitElement {
    static styles = css`
        :host {
            font-family: sans-serif;
        }

        a {
            color: rgb(14, 117, 212);
        }
    `;

    static get properties() {
        return {
            component: { type: String },
            reportId: { type: String },
        };
    }

    constructor() {
        super();
        this.component = "dashboard";
        this.reportId = null;
        window.onhashchange = () => this.onHashChange();
        this.onHashChange();
    }

    async onHashChange() {
        const hash = document.location.hash;
        if (hash == "#/reports") {
            this.component = "reports";
        } else if (hash.startsWith("#/reports/")) {
            this.component = "report";
            this.reportId = hash.substring(10);
        } else {
            this.component = "dashboard";
        }
    }

    render() {
        let component;
        if (this.component == "reports") {
            component = html`<dmarc-reports></dmarc-reports>`;
        } else if (this.component == "report") {
            component = html`<dmarc-report id="${this.reportId}"></dmarc-report>`;
        } else {
            component = html`<dmarc-dashboard></dmarc-dashboard>`;
        }
        return html`
            <p>
                <a href="#/dashboard">Dashboard</a> |
                <a href="#/reports">Reports</a>
            </p>
            ${component}
        `;
    }
}

customElements.define("dmarc-app", App);

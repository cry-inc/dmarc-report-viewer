import { LitElement, html, css } from "lit";

export class Dashboard extends LitElement {
    static styles = css`
        :host {
            display: grid;
            column-gap: 10px;
            row-gap: 10px;
        }

        .module {
            border: 1px solid #aaa;
            border-radius: 3px;
            background-color: #eee;
            padding: 5px;
        }

        .module canvas {
            max-width: 300px;
        }
    `;

    static properties = {
        mails: { type: Number },
        xmlFiles: { type: Number },
        reports: { type: Number },
    };

    constructor() {
        super();

        this.mails = 0;
        this.xmlFiles = 0;
        this.reports = 0;
    }

    async firstUpdated() {
        const response = await fetch("summary");
        const summary = await response.json();

        this.mails = summary.mails;
        this.xmlFiles = summary.xml_files;
        this.reports = summary.reports;

        this.createPieChart("orgs_chart", summary.orgs);
        this.createPieChart("domains_chart", summary.domains);
        this.createPieChart("spf_policy_chart", summary.spf_policy_results);
        this.createPieChart("dkim_policy_chart", summary.dkim_policy_results);
        this.createPieChart("spf_auth_chart", summary.spf_auth_results);
        this.createPieChart("dkim_auth_chart", summary.dkim_auth_results);
    }

    async createPieChart(canvasId, dataMap) {
        const element = this.renderRoot.querySelector("." + canvasId);
        const labels = Object.keys(dataMap);
        const data = labels.map(k => dataMap[k]);
        new Chart(element, {
            type: "pie",
            data: {
                labels,
                datasets: [{ data }]
            }
        });
    }

    render() {
        return html`
            <div class="module" style="grid-column: 1; grid-row: 1;">
                <h2>Inbox</h2>
                <ul>
                    <li>Mails: ${this.mails}</li>
                    <li>XML Files: ${this.xmlFiles}</li>
                    <li>DMARC Reports: ${this.reports}</li>
                </ul>
            </div>

            <div class="module" style="grid-column: 2; grid-row: 1;">
                <h2>Domains</h2>
                <canvas class="domains_chart"></canvas>
            </div>

            <div class="module" style="grid-column: 3; grid-row: 1;">
                <h2>Orgs</h2>
                <canvas class="orgs_chart"></canvas>
            </div>

            <div class="module" style="grid-column: 1; grid-row: 2;">
                <h2>SPF Policy Results</h2>
                <canvas class="spf_policy_chart"></canvas>
            </div>

            <div class="module" style="grid-column: 2; grid-row: 2;">
                <h2>DKIM Policy Results</h2>
                <canvas class="dkim_policy_chart"></canvas>
            </div>

            <div class="module" style="grid-column: 3; grid-row: 2;">
                <h2>SPF Auth Results</h2>
                <canvas class="spf_auth_chart"></canvas>
            </div>

            <div class="module" style="grid-column: 4; grid-row: 2;">
                <h2>DKIM Auth Results</h2>
                <canvas class="dkim_auth_chart"></canvas>
            </div>
        `;
    }
}

customElements.define("dmarc-dashboard", Dashboard);

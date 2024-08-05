import { LitElement, html, css } from "lit";

export class Dashboard extends LitElement {
    static styles = css`
        .container {
            display: grid;
            column-gap: 10px;
            row-gap: 10px;
        }

        .module {
            border: 1px solid #e0e0e0;
            border-radius: 3px;
            background-color: #efefef;
            padding: 5px;
            text-align: center;
        }

        .module canvas {
            max-width: 300px;
            margin: auto;
        }

        .stats {
            margin-bottom: 10px;
        }

        .stats span {
            margin-left: 15px;
            margin-right: 15px;
        }
    `;

    static properties = {
        mails: { type: Number },
        xmlFiles: { type: Number },
        reports: { type: Number },
        lastUpdate: { type: Number },
    };

    constructor() {
        super();

        this.mails = 0;
        this.xmlFiles = 0;
        this.reports = 0;
        this.lastUpdate = 0;
    }

    async firstUpdated() {
        const response = await fetch("summary");
        const summary = await response.json();

        this.mails = summary.mails;
        this.xmlFiles = summary.xml_files;
        this.reports = summary.reports;
        this.lastUpdate = summary.last_update;

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
            <div class="module stats">
                <span>Mails: <b>${this.mails}</b></span>
                <span>XML Files: <b>${this.xmlFiles}</b></span>
                <span>DMARC Reports: <b>${this.reports}</b></span>
                <span>Last Update: <b>${new Date(this.lastUpdate * 1000).toLocaleString()}</b></span>
            </div>

            <div class="container">
                <div class="module" style="grid-column: 1; grid-row: 1;">
                    <h2>Domains</h2>
                    <canvas class="domains_chart"></canvas>
                </div>

                <div class="module" style="grid-column: 2; grid-row: 1;">
                    <h2>Organizations</h2>
                    <canvas class="orgs_chart"></canvas>
                </div>

                <div class="module" style="grid-column: 3; grid-row: 1;">
                    <h2>SPF Policy Results</h2>
                    <canvas class="spf_policy_chart"></canvas>
                </div>

                <div class="module" style="grid-column: 1; grid-row: 2;">
                    <h2>DKIM Policy Results</h2>
                    <canvas class="dkim_policy_chart"></canvas>
                </div>

                <div class="module" style="grid-column: 2; grid-row: 2;">
                    <h2>SPF Auth Results</h2>
                    <canvas class="spf_auth_chart"></canvas>
                </div>

                <div class="module" style="grid-column: 3; grid-row: 2;">
                    <h2>DKIM Auth Results</h2>
                    <canvas class="dkim_auth_chart"></canvas>
                </div>
            </div>
        `;
    }
}

customElements.define("dmarc-dashboard", Dashboard);

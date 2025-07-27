import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";

export class Sources extends LitElement {
    static styles = [globalStyle];

    static properties = {
        sources: { type: Array },
    };

    constructor() {
        super();
        this.sources = [];
        this.updateSources();
    }

    async updateSources() {
        const sourcesResponse = await fetch("sources");
        this.sources = await sourcesResponse.json();
    }

    prepareIssueBadges(issues) {
        // Sort to always have the same badge order
        issues.sort();

        // Convert to nice bades with tool tips
        return issues.map(issue => {
            if (issue === "SpfPolicy") {
                return html`<span class="badge badge-negative help" title="DMARC SPF Policy Issue">SPF Policy</span> `
            } else if (issue === "SpfAuth") {
                return html`<span class="badge badge-negative help" title="DMARC SPF Auth Issue">SPF Auth</span> `
            } else if (issue === "DkimPolicy") {
                return html`<span class="badge badge-negative help" title="DMARC DKIM Policy Issue">DKIM Policy</span> `
            } else if (issue === "DkimAuth") {
                return html`<span class="badge badge-negative help" title="DMARC DKIM Auth Issue">DKIM Auth</span> `
            } else {
                html`<span class="badge badge-negative">${issue}</span> `
            }
        })
    }

    render() {
        return html`
            <h1>DMARC Mail Sources</h1>
            <table>
                <tr>
                    <th>IP Address</th>
                    <th class="help" title="Number of records from reports for this IP">Count</th>
                    <th class="sm-hidden">Domain</th>
                    <th class="xs-hidden help" title="Issues detected in reports from this IP">Issues</th>
                </tr>
                ${this.sources.length !== 0 ? this.sources.map((source) =>
                    html`<tr> 
                        <td>${source.ip}</a></td>
                        <td>${source.count}</td>
                        <td class="sm-hidden">${source.domain}</td>
                        <td class="xs-hidden">${this.prepareIssueBadges(source.issues)}</td>
                    </tr>`
                ) : html`<tr>
                        <td colspan="4">No sources found.</td>
                    </tr>`
            }
            </table>
        `;
    }
}

customElements.define("drv-sources", Sources);

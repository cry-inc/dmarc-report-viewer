import { LitElement, html, css } from "lit";

export class Report extends LitElement {
    static styles = css`
        th {
            text-align: left;
        }

        td, th {
            padding-right: 10px;
        }
    `;

    static get properties() {
        return {
            id: { type: String },
            report: { type: Object, attribute: false }
        };
    }

    constructor() {
        super();
        this.id = null;
        this.report = null;
    }

    async updated(changedProperties) {
        if (changedProperties.has("id") && changedProperties.id !== this.id && this.id) {
            const response = await fetch("reports/" + this.id);
            this.report = await response.json();
            //console.log("Report", this.report);
        }
    }

    render() {
        if (!this.report) {
            return html`No report loaded`;
        }

        return html`
            <table>
                <tr>
                    <th colspan="2">Report Metadata</th>
                </tr>
                <tr>
                    <th>Id</th>
                    <td>${this.report.report_metadata.report_id}</td>
                </tr>
                <tr>
                    <th>Org</th>
                    <td>${this.report.report_metadata.org_name}</td>
                </tr>
                <tr>
                    <th>Records</th>
                    <td>${this.report.record.length}</td>
                </tr>
                <tr>
                    <th>Date Range</th>
                    <td>${new Date(this.report.report_metadata.date_range.begin * 1000).toLocaleString()} -
                        ${new Date(this.report.report_metadata.date_range.end * 1000).toLocaleString()}
                    </td>
                </tr>
                <tr>
                    <th>E-Mail</th>
                    <td>${this.report.report_metadata.email}</td>
                </tr>
                <tr>
                    <th>Extra Contact Info</th>
                    <td>${this.report.report_metadata.extra_contact_info}</td>
                </tr>
                <tr>
                    <th>Error</th>
                    <td>${this.report.report_metadata.error}</td>
                </tr>
                <tr>
                    <th>Version</th>
                    <td>${this.report.version}</td>
                </tr>
                <tr>
                    <th colspan="2">Published Policy</th>
                </tr>
                <tr>
                    <th>Domain</th>
                    <td>${this.report.policy_published.domain}</td>
                </tr>
                <tr>
                    <th>adkim</th>
                    <td>${this.report.policy_published.adkim}</td>
                </tr>
                <tr>
                    <th>aspf</th>
                    <td>${this.report.policy_published.aspf}</td>
                </tr>
                <tr>
                    <th>fo</th>
                    <td>${this.report.policy_published.fo}</td>
                </tr>
                <tr>
                    <th>p</th>
                    <td>${this.report.policy_published.p}</td>
                </tr>
                <tr>
                    <th>pct</th>
                    <td>${this.report.policy_published.pct}</td>
                </tr>
                <tr>
                    <th>sp</th>
                    <td>${this.report.policy_published.sp}</td>
                </tr>
            </table>
        `;
    }
}

customElements.define("dmarc-report", Report);

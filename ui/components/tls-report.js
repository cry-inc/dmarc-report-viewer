import { LitElement, html, nothing } from "lit";
import { globalStyle } from "../style.js";
import { join } from "../utils.js";
import { ipDialog } from "./ip.js";

export class TlsReport extends LitElement {
    static styles = [globalStyle];

    static get properties() {
        return {
            id: { type: String },
            mailId: { type: String, attribute: false },
        };
    }

    constructor() {
        super();
        this.id = null;
        this.mailId = null;
        this.report = null;
    }

    async updated(changedProperties) {
        if (changedProperties.has("id") && changedProperties.id !== this.id && this.id) {
            const response = await fetch("tls-reports/" + this.id);
            const rwi = await response.json();
            this.report = rwi.report;
            this.mailId = rwi.mail_id;
        }
    }

    renderPolicyTypeBadge(result) {
        switch (result) {
            case "no-policy-found":
                return html`<span class="faded">No Policy Found</span>`;
            case "sts":
                return html`<span class="badge">MTA-STS</span>`;
            case "tlsa":
                return html`<span class="badge">TLSA</span>`;
            default:
                return html`<span class="badge">${result}</span>`;
        }
    }

    renderFailureCountBadge(count) {
        if (count === 0) {
            return html`<span class="badge badge-positive">0</span>`;
        } else {
            return html`<span class="badge badge-negative">${count}</span>`;
        }
    }

    renderFailureResultType(type) {
        switch (type) {
            case "starttls-not-supported":
                return html`STARTTLS not supported`;
            case "certificate-host-mismatch":
                return html`Certificate host mismatch`;
            case "certificate-expired":
                return html`Certificate expired`;
            case "certificate-not-trusted":
                return html`Certificate not trusted`;
            case "validation-failure":
                return html`Validation failure`;
            case "tlsa-invalid":
                return html`TLSA invalid`;
            case "dnssec-invalid":
                return html`DNSSEC invalid`;
            case "dane-required":
                return html`DANE required`;
            case "sts-policy-fetch-error":
                return html`MTA-STS policy fetch error`;
            case "sts-policy-invalid":
                return html`MTA-STS policy invalid`;
            case "sts-webpki-invalid":
                return html`MTA-STS WebPKI invalid`;
            default:
                return html`${type}`;
        }
    }

    renderMultilineCell(array) {
        const lines = array.map(l => html`${l}`);
        return join(lines, html`<br>`);
    }

    renderIfDefined(obj) {
        if (obj === undefined) {
            return html`<span class="faded">loading...</span>`;
        } else if (obj) {
            return obj;
        } else {
            return html`<span class="faded">n/a</span>`;
        }
    }

    render() {
        if (!this.report) {
            return html`No report loaded`;
        }

        return html`
            <h1>SMTP TLS Report Details</h1>
            <p class="print-hide">
                <a class="button" href="#/mails/${this.mailId}">Show Mail</a>
                <a class="button" href="tls-reports/${this.id}/json" target="_blank">Open JSON</a>
                <a class="button" href="javascript:window.print()">Print/Save PDF</a>
            </p>
            <table>
                <tr>
                    <th colspan="2">Report Header</td>
                </tr>
                <tr>
                    <td class="name">ID</td>
                    <td>${this.report["report-id"]}</td>
                </tr>
                <tr>
                    <td class="name">Organization</td>
                    <td>${this.report["organization-name"]}</td>
                </tr>
                <tr>
                    <td class="name">Evaluated Policies</td>
                    <td>${this.report.policies.length}</td>
                </tr>
                <tr>
                    <td class="name">Date Range Begin</td>
                    <td>${new Date(this.report["date-range"]["start-datetime"]).toLocaleString()}</td>
                </tr>
                <tr>
                    <td class="name">Date Range End</td>
                    <td>${new Date(this.report["date-range"]["end-datetime"]).toLocaleString()}</td>
                </tr>
                <tr>
                    <td class="name">Contact Info</td>
                    <td>${this.renderIfDefined(this.report["contact-info"])}</td>
                </tr>
            </table>

            ${this.report.policies.sort((a, b) => a.policy["policy-type"].localeCompare(b.policy["policy-type"])).map((policy) => html`
                <h2>Policies</h2>
                <table>
                    <tr>
                        <th colspan="2">Published Policy</td>
                    </tr>
                    <tr>
                        <td class="name">Policy Type</td>
                        <td>${this.renderPolicyTypeBadge(policy.policy["policy-type"])}</td>
                    </tr>
                    ${"policy-string" in policy.policy ? html`
                        <tr>
                            <td class="name">Policy String</td>
                            <td>${this.renderMultilineCell(policy.policy["policy-string"])}</td>
                        </tr>
                    ` : nothing}
                    <tr>
                        <td class="name">Policy Domain</td>
                        <td>${policy.policy["policy-domain"]}</td>
                    </tr>
                    ${"mx-host" in policy.policy ? html`
                        <tr>
                            <td class="name">MX Host</td>
                            <td>${this.renderMultilineCell(policy.policy["mx-host"])}</td>
                        </tr>
                    ` : nothing}
                </table>
                <table>
                    <tr>
                        <th colspan="2">Summary</td>
                    </tr>
                    <tr>
                        <td class="name">Successful Count</td>
                        <td>${policy.summary["total-successful-session-count"]}</td>
                    </tr>
                    <tr>
                        <td class="name">Failure Count</td>
                        <td>${this.renderFailureCountBadge(policy.summary["total-failure-session-count"])}</td>
                    </tr>
                </table>

                ${"failure-details" in policy ? policy["failure-details"].map((failureDetails) => html`
                    <table>
                        <tbody>
                            <tr>
                                <th colspan="2">Failure</td>
                            </tr>
                            <tr>
                                <td class="name">Failure Result Type</td>
                                <td>${this.renderFailureResultType(failureDetails["result-type"])}</td>
                            </tr>
                            <tr>
                                <td class="name">Sending MTA IP</td>
                                <td>
                                    ${this.renderIfDefined(failureDetails["sending-mta-ip"])}
                                    ${failureDetails["sending-mta-ip"] ? html`
                                        <button @click="${() => ipDialog(failureDetails["sending-mta-ip"])}" class="button sm help print-hide" title="Search DNS hostname for IP and geolocate it">DNS and Location</button>
                                        <a class="button sm help print-hide" title="Look up WHOIS record for IP and show in new tab" target="blank" href="ips/${failureDetails["sending-mta-ip"]}/whois">WHOIS</a>
                                    ` : nothing}
                                </td>
                            </tr>
                        </tbody>
                        <tbody>
                            <tr>
                                <td class="name">Receiving MX Host</td>
                                <td>${this.renderIfDefined(failureDetails["receiving-mx-hostname"])}</td>
                            </tr>
                            ${"receiving-mx-helo" in failureDetails ? html`
                                <tr>
                                    <td class="name">Receiving MTA HELO</td>
                                    <td>${failureDetails["receiving-mx-helo"]}</td>
                                </tr>
                            ` : nothing}
                        </tbody>
                        ${"receiving-ip" in failureDetails ? html`
                            <tbody>
                                <tr>
                                    <td class="name">Receiving IP</td>
                                    <td>
                                        ${failureDetails["receiving-ip"]}
                                        <button @click="${() => ipDialog(failureDetails["receiving-ip"])}" class="button sm help print-hide" title="Search DNS hostname for IP and geolocate it">DNS and Location</button>
                                        <a class="button sm help print-hide" title="Look up WHOIS record for IP and show in new tab" target="blank" href="ips/${failureDetails["receiving-ip"]}/whois">WHOIS</a>
                                    </td>
                                </tr>
                            </tbody>
                        ` : nothing}
                        <tbody>
                            <tr>
                                <td class="name">Failed Session Count</td>
                                <td>${failureDetails["failed-session-count"]}</td>
                            </tr>
                            ${"additional-information" in failureDetails ? html`
                                <tr>
                                    <td class="name">Additional Information</td>
                                    <td>${failureDetails["additional-information"]}</td>
                                </tr>
                            ` : nothing}
                            ${"failure-reason-code" in failureDetails ? html`
                                <tr>
                                    <td class="name">Failure Reason Code</td>
                                    <td>${failureDetails["failure-reason-code"]}</td>
                                </tr>
                            ` : nothing}
                        </tbody>
                    `) : nothing}
                </table>
            `)}
        `;
    }
}

customElements.define("drv-tls-report", TlsReport);

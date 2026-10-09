import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";
import { navigate } from "../utils.js";

export class DomainFilter extends LitElement {
    static styles = [globalStyle];

    static properties = {
        route: { type: String },
        params: { type: Object },
        domains: { type: Array },
    };

    constructor() {
        super();
        this.route = "dashboard";
        this.params = {};
        this.domains = [];
    }

    onDomainChange(event) {
        navigate(this.route, this.params, { domain: event.target.value });
    }

    render() {
        // Domains are compared case insensitive, like the backend does
        const selected = (this.params.domain ?? "").toLowerCase();
        return html`
            <label>Domain:
                <select @change="${this.onDomainChange}">
                    <option .selected=${!selected} value="">All</option>
                    ${this.domains.map((domain) => {
                        const value = encodeURIComponent(domain);
                        return html`<option .selected=${selected === value.toLowerCase()} value="${value}">${domain}</option>`;
                    })}
                </select>
            </label>
        `;
    }
}

customElements.define("drv-domain-filter", DomainFilter);

import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";
import { navigate } from "../utils.js";

export class TimeFilter extends LitElement {
    static styles = [globalStyle];

    static properties = {
        route: { type: String },
        params: { type: Object },
    };

    constructor() {
        super();
        this.route = "dashboard";
        this.params = {};
    }

    onTimeSpanChange(event) {
        // The custom dates are only valid together with the custom time span
        navigate(this.route, this.params, { ts: event.target.value, from: "", to: "" });
    }

    onDateChange(key, event) {
        navigate(this.route, this.params, { [key]: event.target.value });
    }

    render() {
        const ts = this.params.ts;
        return html`
            <select aria-label="Time Span" @change="${this.onTimeSpanChange}">
                <option .selected=${!ts} value="">Everything</option>
                <option .selected=${ts === "72"} value="72">Last Three Days</option>
                <option .selected=${ts === "168"} value="168">Last Week</option>
                <option .selected=${ts === "744"} value="744">Last Month</option>
                <option .selected=${ts === "4464"} value="4464">Last Six Months</option>
                <option .selected=${ts === "8760"} value="8760">Last Year</option>
                <option .selected=${ts === "custom"} value="custom">Custom</option>
            </select>
            ${ts === "custom" ? html`
                <label>From <input type="date" .value="${this.params.from ?? ""}"
                    @change="${(e) => this.onDateChange("from", e)}"></label>
                <label>To <input type="date" .value="${this.params.to ?? ""}"
                    @change="${(e) => this.onDateChange("to", e)}"></label>` : ""
            }
        `;
    }
}

customElements.define("drv-time-filter", TimeFilter);

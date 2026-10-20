import { html } from "lit";

export function join(elements, joiner) {
    return elements.flatMap(x => [joiner, x]).slice(1);
}

const timeSpans = [
    ["0", "Everything"],
    ["24", "Last Day"],
    ["72", "Last Three Days"],
    ["168", "Last Week"],
    ["744", "Last Month"],
    ["4464", "Last Six Months"],
    ["8760", "Last Year"],
];

const datePattern = /^\d{4}-\d{2}-\d{2}$/;

function validDate(value) {
    return datePattern.test(value ?? "") ? value : "";
}

function isoDate(date) {
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

// Dates are whole days in local time
export function timeQuery(params) {
    const query = [];
    if (/^\d+$/.test(params.ts ?? "") && params.ts !== "0") {
        query.push("time_span=" + params.ts);
    }
    if (validDate(params.from)) {
        query.push("from=" + Math.floor(new Date(params.from + "T00:00:00").getTime() / 1000));
    }
    if (validDate(params.to)) {
        query.push("to=" + Math.floor(new Date(params.to + "T23:59:59").getTime() / 1000));
    }
    return query;
}

export function timeParams(params) {
    return ["ts", "from", "to"].filter(k => params[k]).map(k => k + "=" + params[k]).join("&");
}

function localDate(value) {
    return new Date(value + "T00:00:00").toLocaleDateString();
}

function rangeText(from, to) {
    if (from && to) {
        return localDate(from) + " – " + localDate(to);
    }
    return from ? "Since " + localDate(from) : "Until " + localDate(to);
}

export function timeSpanSelect(params, route, label) {
    const navigate = (changes) => {
        const newParams = { ...params, ...changes };
        Object.keys(newParams).filter(k => !newParams[k]).forEach(k => delete newParams[k]);
        const query = Object.keys(newParams).map(k => k + "=" + newParams[k]).join("&");
        document.location.href = "#/" + route + (query.length > 0 ? "?" + query : "");
    };
    const from = validDate(params.from);
    const to = validDate(params.to);
    const custom = from || to;
    const selected = custom ? "range" : (params.ts ?? "0");
    const popover = (event) => event.target.getRootNode().querySelector(".time-range");
    const onSelect = (event) => {
        const value = event.target.value;
        if (value === "custom") {
            event.target.value = selected;
            popover(event).showPopover();
        } else {
            navigate({ ts: value === "0" ? "" : value, from: "", to: "" });
        }
    };
    const invalidRange = (fromValue, toValue) => Boolean(fromValue && toValue && toValue < fromValue);
    const onInput = (event) => {
        const box = popover(event);
        const [fromInput, toInput] = box.querySelectorAll("input");
        const invalid = invalidRange(fromInput.value, toInput.value);
        box.querySelector(".apply").disabled = invalid;
        box.querySelector(".error").hidden = !invalid;
    };
    const onApply = (event) => {
        const [fromInput, toInput] = popover(event).querySelectorAll("input");
        navigate({ ts: "", from: fromInput.value, to: toInput.value });
        popover(event).hidePopover();
    };
    const weekAgo = new Date();
    weekAgo.setDate(weekAgo.getDate() - 7);
    const fromValue = custom ? from : isoDate(weekAgo);
    const toValue = custom ? to : isoDate(new Date());
    const invalid = invalidRange(fromValue, toValue);
    const css = label ? "" : "button";
    return html`${label ? label + ": " : ""}<select class="${css}" @change="${onSelect}">
            ${timeSpans.map(([value, text]) =>
                html`<option ?selected=${selected === value} value="${value}">${text}</option>`
            )}
            ${custom ? html`<option selected value="range">${rangeText(from, to)}</option>` : ""}
            <option value="custom">Custom Range...</option>
        </select>
        <div class="time-range" popover @input="${onInput}">
            <label>From <input type="date" .value="${fromValue}"></label>
            <label>To <input type="date" .value="${toValue}"></label>
            <button class="button apply" ?disabled=${invalid} @click="${onApply}">Apply</button>
            <button class="button" @click="${e => popover(e).hidePopover()}">Cancel</button>
            <div class="error" role="alert" ?hidden=${!invalid}>The To date must not be before the From date.</div>
        </div>`;
}

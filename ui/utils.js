export function join(elements, joiner) {
    return elements.flatMap(x => [joiner, x]).slice(1);
}

// Navigates to the route with the changed params, empty values remove a param.
// Expects values that are already URL encoded!
export function navigate(route, params, changes) {
    const merged = { ...params, ...changes };
    const query = Object.keys(merged)
        .filter(k => merged[k])
        .map(k => k + "=" + merged[k])
        .join("&");
    document.location.href = "#/" + route + (query ? "?" + query : "");
}

// Converts the time filter params (ts, from, to) to query params for the backend
export function timeQueryParams(params) {
    const query = [];
    if (params.ts === "custom") {
        // Without time zone the dates are parsed as local time, like the dates shown in the UI
        const from = new Date(params.from + "T00:00:00").getTime();
        const to = new Date(params.to + "T23:59:59").getTime();
        if (!isNaN(from)) query.push("date_from=" + Math.floor(from / 1000));
        if (!isNaN(to)) query.push("date_to=" + Math.floor(to / 1000));
    } else if (parseInt(params.ts) > 0) {
        query.push("date_from=" + Math.floor(Date.now() / 1000 - parseInt(params.ts) * 3600));
    }
    return query;
}

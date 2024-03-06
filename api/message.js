// return a message in JSON format. We need to pass as a parameter
// res, the response we send to the client (Angular). The parameter
// the data is a JavaScript object.
function sendMessage (res, data) {
    res.json ({ status: 'ok', data: data });
}

function sendError (res, reason) {
     res.json ({ status: 'error', data: {reason: reason }});
}

module.exports = { sendMessage, sendError };
const process = require('node:process');
const {spawn} = require('node:child_process');
const {sendError, sendMessage} = require ("./message");

async function Callscript_rule(req, res) {
    const cwd = process.cwd();

    try {
        let result = await new Promise((resolve, reject) => {
            let proc = spawn('/home/chemodots/engine/run.sh', [ 'chemodots-toolkit' ], {
                "env": {},
            });

            let out = '';
            let err = '';

            proc.stdout.on('data', data => {
                out += data;
            });
            proc.stderr.on('data', data => {
                console.log(data.toString());
                err += data;
            });

            proc.on('close', code => {
                if (code !== 0)
                    reject(err);
                else
                    resolve(out);
            });

            proc.stdin.write(JSON.stringify(req.body));
            proc.stdin.end();
        });

        sendMessage(res, JSON.parse(result));
    } catch (e) {
        sendError(res, e.toString());
    }
}
module.exports=Callscript_rule;

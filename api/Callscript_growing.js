const process = require('node:process');
const {spawn} = require('node:child_process');
const {sendError, sendMessage} = require ("./message");

async function Callscript_growing(req, res) {
    const cwd = process.cwd();
    const out_dir = `${cwd}/out`;

    try {
        let result = await new Promise((resolve, reject) => {
            let proc = spawn('/home/chemodots/engine/run.sh', [ 'chemodots-reactor' ], {
                "cwd": out_dir,
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

            let params = JSON.stringify(req.body);

            console.log(`Starting a new Growing job with params: ${params}`);

            proc.stdin.write(params);
            proc.stdin.end();
        });

        console.log(`Got result: ${result}`);

        /*result = {
            info: JSON.parse(result),
            downloads: [
                { title: "Final products (SMILES)", value: `${download_prefix}/focused-db_prods.smi` },
                { title: "Final products (SDF)", value: `${download_prefix}/focused-db_prods.sdf` },
            ],
        };*/
        sendMessage(res, JSON.parse(result));
    } catch (e) {
        sendError(res, e.toString());
    }
}
module.exports=Callscript_growing;

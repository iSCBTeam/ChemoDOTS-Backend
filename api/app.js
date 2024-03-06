const express = require ('express');
const app = express ();
const port = process.env.PORT || 3000;

const bodyParser = require ('body-parser');
app.use(bodyParser.json());

const cors = require ('cors');

const Callscript_func = require ('./Callscript_func');
const Callscript_toolkit = require ('./Callscript_toolkit');
const Callscript_growing = require ('./Callscript_growing');
const Callscript_pp_filter = require ('./Callscript_pp_filter');
const Callscript_pp_gen2d = require ('./Callscript_pp_gen2d');
const Callscript_pp_gen3d = require ('./Callscript_pp_gen3d');

app.post ('/Callscript_func', (req, res) => {Callscript_func(req,res);});
app.post ('/Callscript_toolkit', (req, res) => {Callscript_toolkit(req,res);});
app.post ('/Callscript_growing', (req, res) => {Callscript_growing(req,res);});
app.post ('/Callscript_pp_filter', (req, res) => {Callscript_pp_filter(req,res);});
app.post ('/Callscript_pp_gen2d', (req, res) => {Callscript_pp_gen2d(req,res);});
app.post ('/Callscript_pp_gen3d', (req, res) => {Callscript_pp_gen3d(req,res);});

app.listen(port, () => {console.log (`listening on port ${port}`)});

import * as React from 'react';

import { ThemeProvider, createTheme } from '@mui/material/styles';
import {
  Backdrop,
  Button,
  CircularProgress,
  CssBaseline,
  Container,
  FormControl,
  FormControlLabel,
  FormLabel,
  Link,
  Pagination,
  Paper,
  Radio,
  RadioGroup,
  Typography,
  TextField,
} from '@mui/material';

import Graph from './graph.tsx';

const { useState, useCallback } = React;

const darkTheme = createTheme({
  palette: {
    mode: 'dark',
  },
});

const placeholder = `s(7).
s(4).
s(8).
s(3).
s(5).
s(9).
s(10).
s(1).
s(6).
s(2).
ac(7,c(v)).
ac(4,6).
ac(8,or(neg(1),7)).
ac(3,and(or(7,neg(6)),2)).
ac(5,4).
ac(9,neg(7)).
ac(10,and(neg(2),6)).
ac(1,and(neg(7),2)).
ac(6,neg(7)).ac(2,and(neg(9),neg(6))).`;

enum Parsing {
  Naive = 'Naive',
  Hybrid = 'Hybrid',
}

enum Strategy {
  ParseOnly = 'ParseOnly',
  Ground = 'Ground',
  Complete = 'Complete',
  Stable = 'Stable',
  StableCountingA = 'StableCountingA',
  StableCountingB = 'StableCountingB',
  StableNogood = 'StableNogood',
}

function App() {
  const [loading, setLoading] = useState(false);
  const [code, setCode] = useState(placeholder);
  const [parsing, setParsing] = useState(Parsing.Naive);
  const [graphs, setGraphs] = useState();
  const [graphIndex, setGraphIndex] = useState(0);

  const submitHandler = useCallback(
    (strategy: Strategy) => {
      setLoading(true);

      fetch(`${process.env.NODE_ENV === 'development' ? '//localhost:8080' : ''}/solve`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ code, strategy, parsing }),
      })
        .then((res) => res.json())
        .then((data) => {
          setGraphs(data);
          setGraphIndex(0);
        })
        .finally(() => setLoading(false));
      // TODO: error handling
    },
    [code, parsing],
  );

  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />
      <main>
        <Typography variant="h2" component="h1" align="center" gutterBottom>
          Solve your ADF Problem with OBDDs!
        </Typography>

        <Container>
          <TextField
            name="code"
            label="Put your code here:"
            helperText={(
              <>
                For more info on the syntax, have a
                look
                {' '}
                <Link href="https://github.com/ellmau/adf-obdd" target="_blank" rel="noreferrer">here</Link>
                .
              </>
            )}
            multiline
            fullWidth
            variant="filled"
            value={code}
            onChange={(event) => { setCode(event.target.value); }}
          />
        </Container>
        <Container sx={{ marginTop: 2, marginBottom: 2 }}>
          <FormControl>
            <FormLabel id="parsing-radio-group">Parsing Strategy</FormLabel>
            <RadioGroup
              row
              aria-labelledby="parsing-radio-group"
              name="parsing"
              value={parsing}
              onChange={(e) => setParsing((e.target as HTMLInputElement).value)}
            >
              <FormControlLabel value={Parsing.Naive} control={<Radio />} label="Naive" />
              <FormControlLabel value={Parsing.Hybrid} control={<Radio />} label="Hybrid" />
            </RadioGroup>
          </FormControl>
          <br />
          <br />
          <Button variant="outlined" onClick={() => submitHandler(Strategy.ParseOnly)}>Parse only</Button>
          {' '}
          <Button variant="outlined" onClick={() => submitHandler(Strategy.Ground)}>Grounded Model</Button>
          {' '}
          <Button variant="outlined" onClick={() => submitHandler(Strategy.Complete)}>Complete Models</Button>
          {' '}
          <Button variant="outlined" onClick={() => submitHandler(Strategy.Stable)}>Stable Models (naive heuristics)</Button>
          {' '}
          <Button disabled={parsing !== Parsing.Hybrid} variant="outlined" onClick={() => submitHandler(Strategy.StableCountingA)}>Stable Models (counting heuristic A)</Button>
          {' '}
          <Button disabled={parsing !== Parsing.Hybrid} variant="outlined" onClick={() => submitHandler(Strategy.StableCountingB)}>Stable Models (counting heuristic B)</Button>
          {' '}
          <Button variant="outlined" onClick={() => submitHandler(Strategy.StableNogood)}>Stable Models using nogoods (Simple Heuristic)</Button>
        </Container>

        {graphs
        && (
        <Container sx={{ marginTop: 4, marginBottom: 4 }}>
          {graphs.length > 1
            && (
            <>
              Models:
              <br />
              <Pagination variant="outlined" shape="rounded" count={graphs.length} page={graphIndex + 1} onChange={(e, value) => setGraphIndex(value - 1)} />
            </>
            )}
          {graphs.length > 0
            && (
            <Paper elevation={3} square sx={{ marginTop: 4, marginBottom: 4 }}>
              <Graph graph={graphs[graphIndex]} />
            </Paper>
            )}
          {graphs.length === 0
            && <>No models!</>}
        </Container>
        )}
      </main>

      <Backdrop
        open={loading}
      >
        <CircularProgress color="inherit" />
      </Backdrop>
    </ThemeProvider>
  );
}

export default App;

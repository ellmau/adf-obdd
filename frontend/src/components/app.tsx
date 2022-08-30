import * as React from 'react';

import { ThemeProvider, createTheme } from '@mui/material/styles';
import {
  Backdrop, Button, CircularProgress, CssBaseline, Container, Link, Paper, Typography, TextField,
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

function App() {
  const [loading, setLoading] = useState(false);
  const [code, setCode] = useState(placeholder);
  const [graph, setGraph] = useState();

  const submitHandler = useCallback(
    () => {
      setLoading(true);

      fetch('http://localhost:8080/solve', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ code }),
      })
        .then((res) => res.json())
        .then((data) => setGraph(data))
        .finally(() => setLoading(false));
      // TODO: error handling
    },
    [code],
  );

  console.log(graph);

  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />
      <main>
        <Typography variant="h2" component="h1" align="center" gutterBottom>
          Solve your ADF Problem with Style!
        </Typography>

        <Container>
          <TextField
            name="code"
            label="Put your code here:"
            helperText={(
              <>
                For more info on the syntax, have a look
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
        <Container maxWidth="xs">
          <Button fullWidth variant="outlined" onClick={submitHandler}>Solve it!</Button>
        </Container>

        {graph
        && (
        <Container>
          <Paper elevation={3} square sx={{ marginTop: 4, marginBottom: 4 }}>
            <Graph graph={graph} />
          </Paper>
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

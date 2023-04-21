import React, {
  useState, useContext, useCallback, useRef,
} from 'react';

import {
  Button,
  Container,
  FormControl,
  FormControlLabel,
  FormLabel,
  Link,
  Paper,
  Radio,
  RadioGroup,
  Stack,
  Typography,
  TextField,
  ToggleButtonGroup,
  ToggleButton,
} from '@mui/material';

import LoadingContext from './loading-context';
import SnackbarContext from './snackbar-context';

import { Parsing } from './adf-details';

const PLACEHOLDER = `s(a).
s(b).
s(c).
s(d).
ac(a,c(v)).
ac(b,b).
ac(c,and(a,b)).
ac(d,neg(b)).`;

function AdfNewForm({ fetchProblems }: { fetchProblems: () => void; }) {
  const { setLoading } = useContext(LoadingContext);
  const { setStatus: setSnackbarInfo } = useContext(SnackbarContext);
  const [isFileUpload, setFileUpload] = useState(false);
  const [code, setCode] = useState(PLACEHOLDER);
  const [filename, setFilename] = useState('');
  const [parsing, setParsing] = useState<Parsing>('Naive');
  const [name, setName] = useState('');
  const fileRef = useRef<HTMLInputElement>(null);

  const addAdf = useCallback(
    () => {
      setLoading(true);

      const formData = new FormData();

      if (isFileUpload && fileRef.current) {
        const file = fileRef.current.files?.[0];
        if (file) {
          formData.append('file', file);
        }
      } else {
        formData.append('code', code);
      }

      formData.append('parsing', parsing);
      formData.append('name', name);

      fetch(`${process.env.NODE_ENV === 'development' ? '//localhost:8080' : ''}/adf/add`, {
        method: 'POST',
        credentials: process.env.NODE_ENV === 'development' ? 'include' : 'same-origin',
        body: formData,
      })
        .then((res) => {
          switch (res.status) {
            case 200:
              setSnackbarInfo({ message: 'Successfully added ADF problem!', severity: 'success', potentialUserChange: true });
              fetchProblems();
              break;
            default:
              setSnackbarInfo({ message: 'An error occured while adding the ADF problem.', severity: 'error', potentialUserChange: true });
              break;
          }
        })
        .finally(() => setLoading(false));
    },
    [isFileUpload, code, filename, parsing, name, fileRef.current],
  );

  return (
    <Container>
      <Paper elevation={8} sx={{ padding: 2 }}>
        <Typography variant="h4" component="h2" align="center" gutterBottom>
          Add a new Problem
        </Typography>
        <Container sx={{ marginTop: 2, marginBottom: 2 }}>
          <Stack direction="row" justifyContent="center">
            <ToggleButtonGroup
              value={isFileUpload}
              exclusive
              onChange={(_e, newValue) => { setFileUpload(newValue); setFilename(''); }}
            >
              <ToggleButton value={false}>
                Write by Hand
              </ToggleButton>
              <ToggleButton value>
                Upload File
              </ToggleButton>
            </ToggleButtonGroup>
          </Stack>
        </Container>

        <Container sx={{ marginTop: 2, marginBottom: 2 }}>
          {isFileUpload ? (
            <Stack direction="row" justifyContent="center">
              <Button component="label">
                {(!!filename && fileRef?.current?.files?.[0]) ? `File '${filename.split(/[\\/]/).pop()}' selected! (Click to change)` : 'Upload File'}
                <input hidden type="file" onChange={(event) => { setFilename(event.target.value); }} ref={fileRef} />
              </Button>
            </Stack>
          ) : (
            <TextField
              name="code"
              label="Put your code here:"
              helperText={(
                <>
                  For more info on the syntax, have a
                  look
                  {' '}
                  <Link href="https://github.com/ellmau/adf-obdd" target="_blank" rel="noopener noreferrer">here</Link>
                  .
                </>
              )}
              multiline
              fullWidth
              variant="filled"
              value={code}
              onChange={(event) => { setCode(event.target.value); }}
            />
          )}
        </Container>

        <Container sx={{ marginTop: 2 }}>
          <Stack direction="row" justifyContent="center" spacing={2}>
            <FormControl>
              <FormLabel id="parsing-radio-group">Parsing Strategy</FormLabel>
              <RadioGroup
                row
                aria-labelledby="parsing-radio-group"
                name="parsing"
                value={parsing}
                onChange={(e) => setParsing(((e.target as HTMLInputElement).value) as Parsing)}
              >
                <FormControlLabel value="Naive" control={<Radio />} label="Naive" />
                <FormControlLabel value="Hybrid" control={<Radio />} label="Hybrid" />
              </RadioGroup>
            </FormControl>
            <TextField
              name="name"
              label="Adf Problem Name (optional):"
              variant="standard"
              value={name}
              onChange={(event) => { setName(event.target.value); }}
            />
            <Button variant="outlined" onClick={() => addAdf()}>Add Adf Problem</Button>
          </Stack>
        </Container>
      </Paper>
    </Container>
  );
}

export default AdfNewForm;

import React from 'react';
import ReactMarkdown from 'markdown-to-jsx';
import {
  Box,
  Link,
  Typography,
} from '@mui/material';

const options = {
  overrides: {
    h1: {
      component: Typography,
      props: {
        gutterBottom: true,
        variant: 'h4',
      },
    },
    h2: {
      component: Typography,
      props: { gutterBottom: true, variant: 'h6' },
    },
    h3: {
      component: Typography,
      props: { gutterBottom: true, variant: 'subtitle1' },
    },
    h4: {
      component: Typography,
      props: {
        gutterBottom: true,
        variant: 'caption',
        paragraph: true,
      },
    },
    p: {
      component: Typography,
      props: { paragraph: true, sx: { '&:last-child': { marginBottom: 0 } } },
    },
    a: {
      component: (props: any) => (
        // eslint-disable-next-line react/jsx-props-no-spreading
        <Link target="_blank" rel="noopener noreferrer" {...props} />
      ),
    },
    li: {
      component: (props: any) => (
        <Box component="li" sx={{ mt: 1 }}>
          {/* eslint-disable-next-line react/jsx-props-no-spreading */}
          <Typography component="span" {...props} />
        </Box>
      ),
    },
  },
};

export default function Markdown(props: any) {
  // eslint-disable-next-line react/jsx-props-no-spreading
  return <ReactMarkdown options={options} {...props} />;
}

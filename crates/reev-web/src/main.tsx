// Main application entry point for Reev web interface

import { render } from 'preact';
import { BenchmarkGrid } from './components/BenchmarkGrid';
import './index.css';

// Initialize the application
function App() {
  return (
    <div className="App">
      <BenchmarkGrid />
    </div>
  );
}

// Render the app
const root = document.getElementById('root');
if (root) {
  render(<App />, root);
} else {
  console.error('Root element not found');
}

// Enable hot module replacement in development
if (import.meta.hot) {
  import.meta.hot.accept();
}

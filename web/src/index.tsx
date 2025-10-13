import { render } from "preact";
import { BenchmarkGrid } from "./components/BenchmarkGrid";
import "./style.css";

export function App() {
  return <BenchmarkGrid />;
}

render(<App />, document.getElementById("app"));

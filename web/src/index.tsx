import { render } from "solid-js/web";

import "./index.css";

const App = () => {
  return <p class="bg-ctp-base text-ctp-text">hi floppa</p>;
}

render(() => <App />, document.getElementById("root")!);

import { HashRouter, Route, Routes } from "react-router-dom";
import ResourceView from "./pages/ResourceView"; 
import "./App.css";
import Layout from "./Layout";
import PlanningView from "./pages/PlanningView";

function App() {
  return (
      <HashRouter basename="/">
          <Routes>
            <Route path=":asset" element={<Layout />}>
              <Route path="planning" element={<PlanningView />} />
              <Route path="resources" element={<ResourceView />} />
            </Route>
          </Routes>
      </HashRouter>
  );
}
export default App;

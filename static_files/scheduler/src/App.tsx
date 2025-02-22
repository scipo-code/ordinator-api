import { HashRouter, Route, Routes } from "react-router-dom";
import AssetPage from "./pages/AssetPage"; 
import NotFound from "./pages/NotFound"; 
import "./App.css";

function App() {
  return (
      <HashRouter basename="/">
        <Routes>
          <Route path="/dashboard/:asset" element={<AssetPage />} />
          <Route path="*" element={<NotFound />} />
        </Routes>
      </HashRouter>
  );
}

export default App;

# Standard imports
import os
import sys
import json
import requests
import warnings
import pandas as pd
from pathlib import Path
from dotenv import load_dotenv

load_dotenv()

# Adding directory to Back folder into Python path 
# to enable Python to recognize Back as a module
sys.path.append(str(Path(os.getenv("BASE_FILE_PATH"))))

# Modules imports
from config import settings
from DataProcessing.processing import sap_formatter
from DataProcessing.processing import modifications_detector
from DataProcessing.processing import emails

warnings.filterwarnings("ignore")

session = requests.Session()
session.trust_env = False

def __get_data_from_API(fileName, neededColumns):
    API_URL = settings.api_url
    urlRequest = API_URL + fileName + "?$format=JSON&$select=" + neededColumns

    response = session.get(urlRequest, auth = ('TEPDK-APP-MYPR', 'qNHa]Ay7d_YsjG'))

    if response.status_code == 200:
        data = json.loads(response.text)
        return data['elements']
    else:
        return print("\\n\\n\\n" + fileName + " : ERROR while fetching data from API\\n\\n\\n")

def main():
    ## Environment variables
    ENV = settings.env
    DATA_PATH = settings.data_path


    AFIH_URL_FILE_NAME = "e1p_010_afih"
    AFIH_NEEDED_COLUMNS = "mandt,aufnr,artpr,priok,equnr,bautl,iloan,iloai,anlzu,iwerk,apgrp,gewrk,aning,gauzt,gaueh,inspk,datan,warpl,abnum,wapos,laufn,obknr,revnr,addat,aduhr,sermat"
  
    AFKO_URL_FILE_NAME = "e1p_010_afko"
    AFKO_NEEDED_COLUMNS = "aufnr,gltrp,gstrp,ftrms,gltrs,gstrs,gstri,getri,gltri,ftrmi,ftrmp,plnbez,stlbez,aufpl,aufnt,aufpt,gluzp,gsuzp"
  
    AFRU_URL_FILE_NAME = "e1p_010_afru"
    AFRU_NEEDED_COLUMNS = "mandt,rueck,rmzhl,arbid,werks,iserh,zeier,ismnw,ismne,idaur,idaue,anzma,pernr,aufpl,aufnr,vornr,ofmnw,ofmne,odaur,odaue,smeng"
  
    AFVC_URL_FILE_NAME = "e1p_010_afvc"
    AFVC_NEEDED_COLUMNS = "mandt,aufpl,aplzl,plnfl,vornr,steus,arbid,ltxa1,anzma,anzzl,prznt,larnt,rueck,rmzhl,objnr,spanz,bedid,anlzu,nprio,pspnr,scope,no_disp,arbii,werki,wempf,ablad,sched_end,pernr,oio_hold,tplnr"
  
    AFVV_URL_FILE_NAME = "e1p_010_afvv"
    AFVV_NEEDED_COLUMNS = "mandt,aufpl,aplzl,meinh,dauno,daune,daumi,daume,einsa,einse,arbei,arbeh,mgvrg,ismnw,puffr,pufgs,ntanf,ntanz,ntend,ntenz,bearz,ofmnw,aufkt"
  
    AUFK_URL_FILE_NAME = "e1p_010_aufk"
    AUFK_NEEDED_COLUMNS = "mandt,aufnr,auart,autyp,refnr,erdat,aenam,ktext,ltext,werks,kostv,stort,sowrk,astnr,phas0,phas1,phas2,phas3,idat1,user4,user9,objnr,pspel,erfzeit,aezeit,yyawsc,yyhours,zzgstrp,zzgltrp,zz_olafd,zz_lafd,zz_easd,vaplz"
  
    AUFM_URL_FILE_NAME = "e1p_010_aufm"
    AUFM_NEEDED_COLUMNS = "mandt,mblnr,mjahr,zeile,ablad,aufnr"
  
    IFLOT_URL_FILE_NAME = "e1p_010_iflot"
    IFLOT_NEEDED_COLUMNS = "mandt,tplnr,tplkz,fltyp,tplma,ernam,iwerk,ingrp"
  
    IFLOTX_URL_FILE_NAME = "e1p_010_iflotx"
    IFLOTX_NEEDED_COLUMNS = "mandt,tplnr,spras,pltxt"
  
    ILOA_URL_FILE_NAME = "e1p_010_iloa"
    ILOA_NEEDED_COLUMNS = "mandt,iloan,tplnr,abckz,swerk,stort,beber"
  
    T352R_URL_FILE_NAME = "e1p_010_t352r"
    T352R_NEEDED_COLUMNS = "mandt,iwerk,revnr,revtx,tplnr,aufnr,objnr,revty"
  
    TJ02T_URL_FILE_NAME = "e1p_010_tj02t"
    TJ02T_NEEDED_COLUMNS = "istat,spras,txt30"
  
    TJ30T_URL_FILE_NAME = "e1p_010_tj30t"
    TJ30T_NEEDED_COLUMNS = "mandt,stsma,estat,spras,txt04,txt30"
  
    TJ02_URL_FILE_NAME = "e1p_010_tj02"
    TJ02_NEEDED_COLUMNS = "istat"
  
    TJ20_URL_FILE_NAME = "e1p_010_tj20"
    TJ20_NEEDED_COLUMNS = "mandt,stsma"
  
    TJ30_URL_FILE_NAME = "e1p_010_tj30"
    TJ30_NEEDED_COLUMNS = "mandt,stsma,estat"
    
    
    print("## Fetch AFIH data")
    afih_data = pd.DataFrame.from_dict(__get_data_from_API(AFIH_URL_FILE_NAME, AFIH_NEEDED_COLUMNS)).astype(str)
    afih_data.drop('links', axis=1, inplace=True)
    afih_formatted_data = sap_formatter.format_afih(afih_data)
    print("## Fetch AFKO data")
    afko_data = pd.DataFrame.from_dict(__get_data_from_API(AFKO_URL_FILE_NAME, AFKO_NEEDED_COLUMNS)).astype(str)
    afko_data.drop('links', axis=1, inplace=True)
    afko_formatted_data = sap_formatter.format_afko(afko_data)
    print("## Fetch AFRU data")
    afru_data = pd.DataFrame.from_dict(__get_data_from_API(AFRU_URL_FILE_NAME, AFRU_NEEDED_COLUMNS)).astype(str)
    afru_data.drop('links', axis=1, inplace=True)
    afru_formatted_data = sap_formatter.format_afru(afru_data)
    print("## Fetch AFVC data")
    afvc_data = pd.DataFrame.from_dict(__get_data_from_API(AFVC_URL_FILE_NAME, AFVC_NEEDED_COLUMNS)).astype(str)
    afvc_data.drop('links', axis=1, inplace=True)
    afvc_formatted_data = sap_formatter.format_afvc(afvc_data)
    print("## Fetch AFVV data")
    afvv_data = pd.DataFrame.from_dict(__get_data_from_API(AFVV_URL_FILE_NAME, AFVV_NEEDED_COLUMNS)).astype(str)
    afvv_data.drop('links', axis=1, inplace=True)
    afvv_formatted_data = sap_formatter.format_afvv(afvv_data)
    print("## Fetch AUFK data")
    aufk_data = pd.DataFrame.from_dict(__get_data_from_API(AUFK_URL_FILE_NAME, AUFK_NEEDED_COLUMNS)).astype(str)
    aufk_data.drop('links', axis=1, inplace=True)
    aufk_formatted_data = sap_formatter.format_aufk(aufk_data)
    print("## Fetch AUFM data")
    aufm_data = pd.DataFrame.from_dict(__get_data_from_API(AUFM_URL_FILE_NAME, AUFM_NEEDED_COLUMNS)).astype(str)
    aufm_data.drop('links', axis=1, inplace=True)
    aufm_formatted_data = sap_formatter.format_aufm(aufm_data)
    print("## Fetch IFLOT data")
    iflot_data = pd.DataFrame.from_dict(__get_data_from_API(IFLOT_URL_FILE_NAME, IFLOT_NEEDED_COLUMNS)).astype(str)
    eiflotdata.drop('links', axis=1, inplace=True)
    ebiflotormatted_data = sap_formatter.format_iflot(iflot_data)
    print("## Fetch IFLOTX data")
    iflotx_data = pd.DataFrame.from_dict(__get_data_from_API(IFLOTX_URL_FILE_NAME, IFLOTX_NEEDED_COLUMNS)).astype(str)
    ebiflotxata.drop('links', axis=1, inplace=True)
    ebaniflotxmatted_data = sap_formatter.format_iflotx(iflotx_data)
    print("## Fetch ILOA data")
    iloa_data = pd.DataFrame.from_dict(__get_data_from_API(ILOA_URL_FILE_NAME, ILOA_NEEDED_COLUMNS)).astype(str)
    iloa_data.drop('links', axis=1, inplace=True)
    iloa_formatted_data = sap_formatter.format_iloa(iloa_data)
    print("## Fetch T352R data")
    t352r_data = pd.DataFrame.from_dict(__get_data_from_API(T352R_URL_FILE_NAME, T352R_NEEDED_COLUMNS)).astype(str)
    et352rdata.drop('links', axis=1, inplace=True)
    ebt352rormatted_data = sap_formatter.format_t352r(t352r_data)
    print("## Fetch TJ02T data")
    tj02t_data = pd.DataFrame.from_dict(__get_data_from_API(TJ02T_URL_FILE_NAME, TJ02T_NEEDED_COLUMNS)).astype(str)
    etj02tdata.drop('links', axis=1, inplace=True)
    ebtj02tormatted_data = sap_formatter.format_tj02t(tj02t_data)
    print("## Fetch TJ30T data")
    tj30t_data = pd.DataFrame.from_dict(__get_data_from_API(TJ30T_URL_FILE_NAME, TJ30T_NEEDED_COLUMNS)).astype(str)
    etj30tdata.drop('links', axis=1, inplace=True)
    ebtj30tormatted_data = sap_formatter.format_tj30t(tj30t_data)
    print("## Fetch TJ02 data")
    tj02_data = pd.DataFrame.from_dict(__get_data_from_API(TJ02_URL_FILE_NAME, TJ02_NEEDED_COLUMNS)).astype(str)
    tj02_data.drop('links', axis=1, inplace=True)
    tj02_formatted_data = sap_formatter.format_tj02(tj02_data)
    print("## Fetch TJ20 data")
    tj20_data = pd.DataFrame.from_dict(__get_data_from_API(TJ20_URL_FILE_NAME, TJ20_NEEDED_COLUMNS)).astype(str)
    tj20_data.drop('links', axis=1, inplace=True)
    tj20_formatted_data = sap_formatter.format_tj20(tj20_data)
    print("## Fetch TJ30 data")
    tj30_data = pd.DataFrame.from_dict(__get_data_from_API(TJ30_URL_FILE_NAME, TJ30_NEEDED_COLUMNS)).astype(str)
    tj30_data.drop('links', axis=1, inplace=True)
    tj30_formatted_data = sap_formatter.format_tj30(tj30_data)

    
    print("## Merge all tables")
    merged_data = sap_formatter.merge_data(
        afih_formatted_data,
        afko_formatted_data,
        afru_formatted_data,
        afvc_formatted_data,
        afvv_formatted_data,
        aufk_formatted_data,
        aufm_formatted_data,
        iflot_formatted_data,
        iflotx_formatted_data,
        iloa_formatted_data,
        t352r_formatted_data,
        tj02t_formatted_data,
        tj30t_formatted_data,
        tj02_formatted_data,
        tj20_formatted_data,
        tj30_formatted_data,
        tj30_formatted_data,
    )
    
    # Processing the data & sort it
    print("## Process data")
    processed_data = sap_formatter.process_data(merged_data).sort_values(
        by=["Purchase Requisition_EBAN", "Item of requisition_EBAN"]
    )

    # Replace the old data processed file
    os.replace(f"{DATA_PATH}\\data_processed_api.csv", f"{DATA_PATH}\\data_processed_api_prev.csv")
    processed_data.to_csv(f"{DATA_PATH}\\data_processed_api.csv")
        

if "__main__" == __name__:
    main()

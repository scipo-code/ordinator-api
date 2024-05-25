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

    EBAN_URL_FILE_NAME = "e1p_010_eban"
    EBAN_NEEDED_COLUMNS = "banfn,bnfpo,loekz,statu,frgzu,ernam,afnam,txz01,matnr,werks,menge,lpein,lfdat,frgdt,ebeln,ebelp,frgrl,waers,banpr,rlwrt,blckd,blckt,yybuyer,preis"
    EKPO_URL_FILE_NAME = "e1p_010_ekpo"
    EKPO_NEEDED_COLUMNS = "ebeln,ebelp,loekz,statu,txz01,matnr,werks,menge,anfnr,banfn,bnfpo,afnam,netpr,peinh"
    EKKN_URL_FILE_NAME = "e1p_010_ekkn"
    EKKN_NEEDED_COLUMNS = "ebeln,ebelp,loekz,menge,kostl,aufnr,ps_psp_pnr"
    EKET_URL_FILE_NAME = "e1p_010_eket"
    EKET_NEEDED_COLUMNS = "ebeln,ebelp,etenr,eindt,lpein,menge,banfn,bnfpo"
    EKBE_URL_FILE_NAME = "e1p_010_ekbe"
    EKBE_NEEDED_COLUMNS = "ebeln,ebelp,vgabe,bwart,menge,waers,cpudt,matnr,werks,bldat,ernam,belnr"
    EKKO_URL_FILE_NAME = "e1p_010_ekko"
    EKKO_NEEDED_COLUMNS = "ebeln,bukrs,loekz,statu,ernam,waers,frgzu,frgrl,rlwrt,yybuyer,bedat"
    EBKN_URL_FILE_NAME = "e1p_010_ebkn"
    EBKN_NEEDED_COLUMNS = "banfn,bnfpo,loekz,ernam,menge,kostl,aufnr,ps_psp_pnr"
    WBS_URL_FILE_NAME = "e1p_010_prps"
    WBS_NEEDED_COLUMNS = "pspnr,posid,ernam,werks,kostl,matnr"
    IGG_URL_FILE_NAME = "e1p_010_ybw_adrp"
    IGG_NEEDED_COLUMNS = "bname,name_text"
    AUFK_URL_FILE_NAME = "e1p_010_aufk"
    AUFK_NEEDED_COLUMNS = "aufnr,ernam,ktext,bukrs,werks,waers,loekz,kostl,objnr"
    IHPA_URL_FILE_NAME = "e1p_010_ihpa"
    IHPA_NEEDED_COLUMNS = "objnr,parvw,counter,parnr"

    ##### Data Formatting
    ## EBAN : Main table : all PR/PO links
    print("## Fetch EBAN data")
    eban_data = pd.DataFrame.from_dict(__get_data_from_API(EBAN_URL_FILE_NAME, EBAN_NEEDED_COLUMNS)).astype(str)
    eban_data.drop('links', axis=1, inplace=True)
    eban_formatted_data = sap_formatter.format_eban(eban_data)

    ## EKET : POs general 
    print("## Fetch EKET data")
    eket_data = pd.DataFrame.from_dict(__get_data_from_API(EKET_URL_FILE_NAME, EKET_NEEDED_COLUMNS)).astype(str)
    eket_data.drop('links', axis=1, inplace=True)
    eket_formatted_data = sap_formatter.format_eket(eket_data)

    ## EKKO : POs Header
    print("## Fetch EKKO data")
    ekko_data = pd.DataFrame.from_dict(__get_data_from_API(EKKO_URL_FILE_NAME, EKKO_NEEDED_COLUMNS)).astype(str)
    ekko_data.drop('links', axis=1, inplace=True)
    ekko_buyer_data = ekko_data.copy()
    ekko_formatted_data = sap_formatter.format_ekko(ekko_data)

    ## EKKO : POs buyers
    print("## Fetch EKKO Buyer data")
    ekko_buyer_formatted_data = sap_formatter.format_ekko_buyer(ekko_buyer_data)

    ## EKPO : Item of POs
    print("## Fetch EKPO data")
    ekpo_data = pd.DataFrame.from_dict(__get_data_from_API(EKPO_URL_FILE_NAME, EKPO_NEEDED_COLUMNS)).astype(str)
    ekpo_data.drop('links', axis=1, inplace=True)
    ekpo_formatted_data = sap_formatter.format_ekpo(ekpo_data)

    ## EKBE : Status of POs
    print("## Fetch EKBE data")
    ekbe_data = pd.DataFrame.from_dict(__get_data_from_API(EKBE_URL_FILE_NAME, EKBE_NEEDED_COLUMNS)).astype(str)
    ekbe_data.drop('links', axis=1, inplace=True)
    ekbe_formatted_data = sap_formatter.format_ekbe(ekbe_data)

    ## EBKN : general data about WOs
    ## WBS : general data about WBS : WO/WBS links -> real name : PRPS
    print("## Fetch EKBN data")
    ebkn_data = pd.DataFrame.from_dict(__get_data_from_API(EBKN_URL_FILE_NAME, EBKN_NEEDED_COLUMNS)).astype(str)
    ebkn_data.drop('links', axis=1, inplace=True)
    wbs_data = pd.DataFrame.from_dict(__get_data_from_API(WBS_URL_FILE_NAME, WBS_NEEDED_COLUMNS)).astype(str)
    wbs_data.drop('links', axis=1, inplace=True)
    ebkn_formatted_data = sap_formatter.format_ebkn_wbs(ebkn_data, wbs_data)

    ## AUFK : WOs description
    print("## Fetch AUFK data")
    aufk_data = pd.DataFrame.from_dict(__get_data_from_API(AUFK_URL_FILE_NAME, AUFK_NEEDED_COLUMNS)).astype(str)
    aufk_data.drop('links', axis=1, inplace=True)
    aufk_formatted_data = sap_formatter.format_aufk(aufk_data)

    ## EKKN : Cost centers
    print("## Fetch EKKN data")
    ekkn_data = pd.DataFrame.from_dict(__get_data_from_API(EKKN_URL_FILE_NAME, EKKN_NEEDED_COLUMNS)).astype(str)
    ekkn_data.drop('links', axis=1, inplace=True)
    ekkn_formatted_data = sap_formatter.format_ekkn(ekkn_data)

    ## IHPA : Partners 
    print("## Fetch IHPA data")
    ihpa_data = pd.DataFrame.from_dict(__get_data_from_API(IHPA_URL_FILE_NAME, IHPA_NEEDED_COLUMNS)).astype(str)
    ihpa_data.drop('links', axis=1, inplace=True)
    ihpa_formatted_data = sap_formatter.format_ihpa(ihpa_data)

    ## YBW_ADRP : Links IGGs of buyer to personal data (name)
    print("## Fetch YBW_ADRP data")
    igg_data = pd.DataFrame.from_dict(__get_data_from_API(IGG_URL_FILE_NAME, IGG_NEEDED_COLUMNS)).astype(str)
    igg_data.drop('links', axis=1, inplace=True)
    igg_formatted_data = sap_formatter.format_igg(igg_data)

    # Merging the data
    print("## Merge all tables")
    merged_data = sap_formatter.merge_data(
        eban_formatted_data,
        ekpo_formatted_data,
        eket_formatted_data,
        ekkn_formatted_data,
        ekbe_formatted_data,
        ekko_formatted_data,
        ebkn_formatted_data,
        ekko_buyer_formatted_data,
        igg_formatted_data,
        aufk_formatted_data,
        ihpa_formatted_data
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

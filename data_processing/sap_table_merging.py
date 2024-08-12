import pandas as pd

def read_csv_file(file_name, columns_dict):
    df =  pd.read_csv('./data/E1P_010_' + file_name + '.csv', delimiter = ';', usecols = columns_dict.keys())
    return df.rename(columns=columns_dict)

def write_mid_level_csv_file(df, file_name):
    df.to_csv('./data/mid_' + file_name + '.csv' if '.csv' not in file_name else '')k




afih_columns = {
    "MANDT" : "MANDT_AFIH",
    "AUFNR" : "AUFNR_AFIH",
    "ARTPR" : "ARTPR_AFIH",
    "PRIOK" : "PRIOK_AFIH",
    "EQUNR" : "EQUNR_AFIH",
    "BAUTL" : "BAUTL_AFIH",
    "ILOAN" : "ILOAN_AFIH",
    "ILOAI" : "ILOAI_AFIH",
    "ANLZU" : "ANLZU_AFIH",
    "IWERK" : "IWERK_AFIH",
    "APGRP" : "APGRP_AFIH",
    "GEWRK" : "GEWRK_AFIH",
    "ANING" : "ANING_AFIH",
    "GAUZT" : "GAUZT_AFIH",
    "GAUEH" : "GAUEH_AFIH",
    "INSPK" : "INSPK_AFIH",
    "DATAN" : "DATAN_AFIH",
    "WARPL" : "WARPL_AFIH",
    "ABNUM" : "ABNUM_AFIH",
    "WAPOS" : "WAPOS_AFIH",
    "LAUFN" : "LAUFN_AFIH",
    "OBKNR" : "OBKNR_AFIH",
    "REVNR" : "REVNR_AFIH",
    "ADDAT" : "ADDAT_AFIH",
    "ADUHR" : "ADUHR_AFIH",
    "SERMAT" : "SERMA_AFIHT",
}

afko_columns = {
    "AUFNR": "AUFNR_AFKO",    
    "GLTRP": "GLTRP_AFKO",    
    "GSTRP": "GSTRP_AFKO",    
    "FTRMS": "FTRMS_AFKO",    
    "GLTRS": "GLTRS_AFKO",    
    "GSTRS": "GSTRS_AFKO",    
    "GSTRI": "GSTRI_AFKO",    
    "GETRI": "GETRI_AFKO",    
    "GLTRI": "GLTRI_AFKO",    
    "FTRMI": "FTRMI_AFKO",    
    "FTRMP": "FTRMP_AFKO",    
    "PLNBEZ": "PLNBEZ_AFKO",    
    "STLBEZ": "STLBEZ_AFKO",    
    "AUFPL": "AUFPL_AFKO",    
    "AUFNT": "AUFNT_AFKO",    
    "AUFPT": "AUFPT_AFKO",
}

afru_columns = {
    
    
"MANDT": "MANDT_AFRU",    
"RUECK": "RUECK_AFRU",    
"RMZHL": "RMZHL_AFRU",    
"ARBID": "ARBID_AFRU",    
"WERKS": "WERKS_AFRU",    
"ISERH": "ISERH_AFRU",    
"ZEIER": "ZEIER_AFRU",    
"ISMNW": "ISMNW_AFRU",    
"ISMNE": "ISMNE_AFRU",    
"IDAUR": "IDAUR_AFRU",    
"IDAUE": "IDAUE_AFRU",    
"ANZMA": "ANZMA_AFRU",    
"PERNR": "PERNR_AFRU",    
"AUFPL": "AUFPL_AFRU",    
"AUFNR": "AUFNR_AFRU",    
"VORNR": "VORNR_AFRU",    
"OFMNW": "OFMNW_AFRU",    
"OFMNE": "OFMNE_AFRU",    
"ODAUR": "ODAUR_AFRU",    
"ODAUE": "ODAUE_AFRU",    
"SMENG": "SMENG_AFRU",}
afvc_columns = 
afvv_columns = 
aufk_columns = 
aufm_columns = 
iflotx_columns = 
iloa_columns = 
t352r_columns = 
tj02_columns = 
tj02t_columns = 
tj20_columns = 
tj30_columns = 
tj30t_columns = 

# Dictionnary of the 58 Rules
# Chemical functions are associated with the rules

# P.S. Some rules do not have any function associated
#      Some functions are not associated to a rule
#      In that case, the 58 rules will be displayed

import moieties

rules = {}
rules["1 : Pictet-Sengler"]=[moieties.aldehyde_alkyl,moieties.aldehyde_aryl,moieties.ketone_alkyl,moieties.ketone_aryl,moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl]
rules["2 : Benzimidazole carboxylic-acid/ester"]=[moieties.ester_alkyl,moieties.ester_aryl,moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl]
rules["3 : Benzimidazole aldehyde"]=[moieties.aldehyde_alkyl,moieties.aldehyde_aryl,moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl]
rules["4 : Benzothiazole"]=[moieties.aldehyde_alkyl,moieties.aldehyde_aryl]
rules["5 : Benzoxazole arom-aldehyde"]=[moieties.aldehyde_alkyl,moieties.aldehyde_aryl]
rules["6 : Benzoxazole carboxylic-acid"]=[]
rules["7 : Thiazole"]=[moieties.ketone_alkyl,moieties.ketone_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I,moieties.ketone_alpha_halide,moieties.ketone_beta_halide,moieties.thioamide_alkyl1,moieties.thioamide_alkyl2]
rules["8 : Niementowski quinazoline"]=[moieties.amide1,moieties.amide2,moieties.amide_alkyl,moieties.amide_aryl]
rules["9 : Tetrazole 1-reactant"]=[moieties.nitrile_alkyl,moieties.nitrile_aryl]
rules["10 : Tetrazole add-step azide-1"]=[moieties.azide_alkyl,moieties.azide_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I,moieties.nitrile_alkyl,moieties.nitrile_aryl]
rules["11 : Tetrazole add-step azide-2"]=[moieties.azide_alkyl,moieties.azide_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I,moieties.nitrile_alkyl,moieties.nitrile_aryl]
rules["12 : Triazole-123 Huisgen add-step azide-1"]=[moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I,moieties.alcohol,moieties.alcohol_alkyl,moieties.alcohol_aryl,moieties.alkyne_alkyl,moieties.alkyne_aryl,moieties.azide_alkyl,moieties.azide_aryl]
rules["13 : Triazole-123 Huisgen add-step azide-2"]=[moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I,moieties.alcohol,moieties.alcohol_alkyl,moieties.alcohol_aryl,moieties.alkyne_alkyl,moieties.alkyne_aryl,moieties.azide_alkyl,moieties.azide_aryl]
rules["14 : Triazole-123 Huisgen add-step azide-3"]=[moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I,moieties.alcohol,moieties.alcohol_alkyl,moieties.alcohol_aryl,moieties.alkyne_alkyl,moieties.alkyne_aryl,moieties.azide_alkyl,moieties.azide_aryl]
rules["15 : Triazole-124 acetohydrazide"]=[moieties.nitrile_alkyl,moieties.nitrile_aryl]
rules["16 : Triazole-124 carboxylic-acid/ester add-step hydrazine"]=[moieties.ester_alkyl,moieties.ester_aryl,moieties.nitrile_alkyl,moieties.nitrile_aryl]
rules["17 : Pyridine-nitrile 1reactant"]=[moieties.dicarbonyl_1_3_alkyl,moieties.dicarbonyl_1_3_aryl,moieties.dicarbonyl_1_4_alkyl,moieties.dicarbonyl_1_4_aryl]
rules["18 : Spiro-chromanone"]=[]
rules["19 : Pyrazole"]=[moieties.dicarbonyl_1_3_alkyl,moieties.dicarbonyl_1_3_aryl,moieties.dicarbonyl_1_4_alkyl,moieties.dicarbonyl_1_4_aryl,moieties.hydrazine_alkyl,moieties.hydrazine_aryl]
rules["20 : Phthalazinone"]=[moieties.hydrazine_alkyl,moieties.hydrazine_aryl]
rules["21 : Pyrrole Paal-Knorr"]=[moieties.hydrazine_alkyl,moieties.hydrazine_aryl,moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl,moieties.dicarbonyl_1_3_alkyl,moieties.dicarbonyl_1_3_aryl,moieties.dicarbonyl_1_4_alkyl,moieties.dicarbonyl_1_4_aryl]
rules["22 : Triaryl imidazole"]=[moieties.aldehyde_alkyl,moieties.aldehyde_aryl,moieties.dicarbonyl_1_3_alkyl,moieties.dicarbonyl_1_3_aryl,moieties.dicarbonyl_1_4_alkyl,moieties.dicarbonyl_1_4_aryl]
rules["23 : Indole Fischer"]=[moieties.aldehyde_alkyl,moieties.aldehyde_aryl,moieties.ketone_alkyl,moieties.ketone_aryl,moieties.hydrazine_alkyl,moieties.hydrazine_aryl]
rules["24 : Quinoline Friedlaender"]=[moieties.aldehyde_alkyl,moieties.aldehyde_aryl,moieties.ketone_alkyl,moieties.ketone_aryl]
rules["25 : Benzofuran"]=[moieties.alkyne_alkyl,moieties.alkyne_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["26 : Benzothiophen"]=[moieties.alkyne_alkyl,moieties.alkyne_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["27 : Indole"]=[moieties.alkyne_alkyl,moieties.alkyne_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["28 : Oxadiazole add-step amidoxime"]=[moieties.nitrile_alkyl,moieties.nitrile_aryl]
rules["29 : Ether Williamson"]=[moieties.alcohol,moieties.alcohol_alkyl,moieties.alcohol_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["30 : Reductive Amination"]=[moieties.aldehyde_alkyl,moieties.aldehyde_aryl,moieties.ketone_alkyl,moieties.ketone_aryl,moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl]
rules["31 : Suzuki"]=[moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["32 : Indole Piperidine"]=[]
rules["33 : Negishi add-step zn-halide"]=[moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["34 : Imide Mitsonobu"]=[moieties.alcohol,moieties.alcohol_alkyl,moieties.alcohol_aryl,moieties.imide_alkyl,moieties.imide_aryl]
rules["35 : Phenol-ether Mitsonobu"]=[moieties.alcohol,moieties.alcohol_alkyl,moieties.alcohol_aryl]
rules["36 : Sulfonamide Mitsonobu"]=[moieties.alcohol,moieties.alcohol_alkyl,moieties.alcohol_aryl,moieties.sulfonamide_alkyl,moieties.sulfonamide_aryl]
rules["37 : Tetrazole Mitsonobu 1"]=[moieties.alcohol,moieties.alcohol_alkyl,moieties.alcohol_aryl]
rules["38 : Tetrazole Mitsonobu 2"]=[moieties.alcohol,moieties.alcohol_alkyl,moieties.alcohol_aryl]
rules["39 : Tetrazole Mitsonobu 3"]=[moieties.alcohol,moieties.alcohol_alkyl,moieties.alcohol_aryl]
rules["40 : Tetrazole Mitsonobu 4"]=[moieties.alcohol,moieties.alcohol_alkyl,moieties.alcohol_aryl]
rules["41 : Vinyl-terminal Heck"]=[moieties.alkene_alkyl,moieties.alkene_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["42 : Vinyl Heck"]=[moieties.alkene_alkyl,moieties.alkene_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["43 : Stille add-step organo-stannane"]=[moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["44 : Carbonyl Grignard add-step magnesium-halide"]=[moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I,moieties.nitrile_alkyl,moieties.nitrile_aryl]
rules["45 : Alcohol Grignard add-step magnesium-halide"]=[moieties.aldehyde_alkyl,moieties.aldehyde_aryl,moieties.ketone_alkyl,moieties.ketone_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["46 : Alkyne Sonogashira"]=[moieties.alkyne_alkyl,moieties.alkyne_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["47 : Amide Schotten-Baumann add-step acyl-chloryde"]=[moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl]
rules["48 : Sulfonamide"]=[moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I,moieties.sulfonylhalide_alkyl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.sulfonylhalide_aryl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["49 : N-arylation heterocycles"]=[]
rules["50 : Wittig add-step ylide-triaryl-phosphine"]=[moieties.aldehyde_alkyl,moieties.aldehyde_aryl,moieties.ketone_alkyl,moieties.ketone_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["51 : Amination Buchwald-Hartwig"]=[moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["52 : Imidazole"]=[moieties.ketone_alkyl,moieties.ketone_aryl,moieties.amidine_alkyl,moieties.amidine_aryl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I,moieties.ketone_alpha_halide,moieties.ketone_beta_halide]
rules["53 : Decarboxylative-coupling"]=[moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["54 : Amination nucleophilic-subst heteroaromatic"]=[moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["55 : Amination nucleophilic-subst aromatic-nitro-ortho"]=[moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["56 : Amination nucleophilic-subst aromatic-nitro-para"]=[moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl,moieties.halide_alkyl_Cl,moieties.halide_alkyl_Br,moieties.halide_alkyl_I,moieties.halide_aryl_Cl,moieties.halide_aryl_Br,moieties.halide_aryl_I]
rules["57 : Urea"]=[moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl,moieties.isocyanate_alkyl,moieties.isocyanate_aryl]
rules["58 : Thiourea"]=[moieties.amine_prim_alkyl,moieties.amine_prim_aryl,moieties.amine_sec_aryl,moieties.amine_sec_alkyl,moieties.amine_tert_aryl,moieties.amine_tert_alkyl]


def displayRules(fctName):
    found=False
    for key,values in rules.items():
        if fctName in rules[key]:
            found=True
            print(key)
            
        
    
    """print( "La clé '{0}' contient la valeur '{1}'.".format(key,values))"""
import sys
if __name__== '__main__':
   Namefct=sys.argv[1]
   displayRules(Namefct)


      
